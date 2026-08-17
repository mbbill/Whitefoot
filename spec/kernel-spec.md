# Kernel Specification v0.31

Status: CANDIDATE v0.31 supersedes v0.30 5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1 (2026-08-17; spec delta: rules +1 [SET-2], operations +1 `buffer_vacant`, tokens +2 `replace` `buffer_vacant`, spellings +1 `replace_let_rhs`, exceptions ±0 — the OWN-5 replace admission is a positive commit rule; evidence-selected: the recorded §5 take/replace collection blocker and the batch-0070 growable-vector, byte-string, and affine-element consumers).
Prior versions: the immutable `spec/kernel-spec-vN.md` archives and the `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification; their derivation status and open evidence are recorded in `spec/derivation/derivation-ledger.md` and relevant live `mcts_mem/` decisions): ordinary loop form (GRAM-4/6; the counted `for_stmt` is evidence-selected in v0.25 and is not this register item), statement-only match (GRAM-7), boundary annotation surface (TYPE-5), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), deref prefix places (GRAM-5), and the `requires { requires_entry* }` surface spelling with its FN-8-checked ordinary-let/final-check subset (FN-8 — semantics selected, spelling not yet compared).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14).
The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is accepted iff it parses under the canonical grammar, satisfies every rule in this document, and every unproven D1-critical checkable fact (bounds; alias-disjointness where a check form exists) carries a runtime check.
There is no writer-emittable third state: nothing writer-stated is trusted unchecked.
The sole trusted-assertion class is toolchain-gated ledger entries (§14), which the writer cannot author or edit.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code.
This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.

[SCOPE-4] A runtime contract violation traps.
Before aborting, the runtime attempts to write the exact [DIAG-3] trap record for the failing checked site to standard error.
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

The line-bearing simple productions are `field`, `variant`, `fn_sig`, `law`, `fn_bind`, `const_decl`, `doc`, `set_stmt`, `expr_stmt`, `return_stmt`, `break_stmt`, `check_stmt`, `claim_stmt`, and `give_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs`, `propagate_let_rhs`, or `replace_let_rhs`.
Each renders completely on one line, including its final semicolon.

The block-bearing productions are `struct_decl`, `enum_decl`, `contract_decl`, `conform_decl`, the body of `fn_decl`, `requires_block`, `ensures_block`, `loop_stmt`, `for_stmt`, `region_stmt`, `match_stmt`, `value_match`, `if_stmt`, `value_if`, and `arm`.
Their introducer through `{` is one line; their children render on following lines at depth plus one; and `}` renders on its own line at the original depth.
Empty blocks still use an opening line followed by a closing-brace line.
An `if_stmt` or `value_if` is rendered solely by this sentence, the generic block-bearing rendering notwithstanding: its introducer through the then-block `{` is one line; then-children render at depth plus one; an `else` renders as the join line `} else {` at the original depth, and a chained `else if` as the join line `} else if` through that `if`'s `{` at the original depth, never as a nested introducer line; else-children render at depth plus one; and the final `}` renders on its own line at the original depth.
No one-line `if` form exists.
A value-match or value-if let places its complete let prefix and the `match` or `if` introducer through `{` on one line.

A function with neither clause block puts its complete header through the body `{` on one line.
A function whose first clause is `requires_block` puts its header through `requires {` on one line; one whose only clause is `ensures_block` instead puts its header through `ensures`, the complete `ensures_selector`, and `{` on one line.
After a `requires_block`, render either its close and the following `ensures_block` introducer as the single line `} ensures ` through that block's `{`, or its close and the body open as the single line `} {`.
After an `ensures_block`, render its close and the body open as the single line `} {`.
Then render the body children and closing brace.
Every production not listed as line-bearing or block-bearing introduces no formatting boundary of its own.
Its terminals stay on the current line unless a descendant line-bearing or block-bearing production introduces one of the boundaries prescribed above.
No other LF or blank line is emitted.

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*` excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)` (single token; the base has the raw lowercase-word shape used by IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a valid field-access place `p.field`: all five suffix words are reserved from field binding [OP-1, GRAM-5]; e.g. `ineg.checked`).

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
STRING appears only in `doc` entries, `check` messages, and `claim` justifications; non-ASCII diagnostic text is DEFERRED.
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
Ruling of record: the exclusive borrow mode is `uniq` (uniqueness-type lineage), not `mut` (Rust divergence: exclusivity is the invariant; mutation is only its permission, and the name breaks under future interior-mutability capabilities).
DEFERRED with recorded delta: the two-axis mode vocabulary (exclusivity x write-permission, adding frozen/exclusive-read and capability-gated shared-write).

## 3. Grammar

[GRAM-1] The grammar is deterministic and unambiguous.
Raw lexical formation scans each source independently from byte offset zero and partitions it into tokens and trivia without normalization, decoding a value, or consulting grammar position, name lookup, the operation table, or another source.
At each cursor it takes exactly the following maximal form; no token or trivia crosses a source boundary.

- One or more ASCII space bytes form one trivia item.
One LF byte forms one trivia item.
- A lower word starts with `[a-z]` and continues through the maximal `[a-z0-9_]*` suffix.
If that complete base is followed immediately by `.` and exactly one of `wrap`, `trap`, `checked`, `sat`, or `strict`, and the suffix is not followed by an ASCII letter, ASCII digit, or `_`, the base, dot, and suffix instead form one operation-name token.
Otherwise the lower word ends before the dot.
- An upper word starts with `[A-Z]` and continues through the maximal `[A-Za-z0-9]*` suffix.
- A region form starts with `'` and a label form starts with `@`; the sigil must be followed by `[a-z]`, after which the token continues through the maximal `[a-z0-9_]*` suffix.
- A numeric form starts with a decimal digit, or with `-` immediately followed by a decimal digit.
It then consumes the maximal sequence of ASCII letters, ASCII digits, `_`, and `.`, plus a `+` or `-` only when that sign byte immediately follows `e` or `E`, except that when the next two bytes are `..` the numeric form ends immediately before the first dot.
A single dot and every other numeric candidate retain the preceding maximal rule unchanged.
Raw formation deliberately retains broad candidates such as `1e+`, `1.00_f64`, and `1.0E2_f64`; [FORM-5] and [FORM-7] decide membership and canonicality without rescanning or splitting them.
- An operator form starts with `+`, `*`, `/`, or `%`, or with a `-` that is immediately followed by neither a decimal digit (numeric form, unchanged) nor `>` (the `->` compound, unchanged), and continues through the maximal `[a-z]*` suffix; the suffix must be empty or one of `wrap`, `checked`, `sat` per the closed `infix_op` list, and any other suffix is a terminal-membership rejection.
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
                "->" rtype effects requires_block? ensures_block? "{" doc? stmt* "}"
program_kind := IDENT
requires_block:= "requires" "{" requires_entry* "}"
requires_entry:= doc | stmt
ensures_block:= "ensures" ensures_selector "{" ensures_entry* "}"
ensures_selector:= IDENT | TYPEID "(" fieldbind_list? ")"
ensures_entry:= doc | stmt
contract_decl:= "contract" TYPEID generics? "{" doc? fn_sig* law* "}"
fn_sig       := "fn" IDENT region_params? "(" param_list? ")" "->" rtype effects ";"
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
input_label  := IDENT "." IDENT "as"
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
             | for_stmt | break_stmt | region_stmt | check_stmt | claim_stmt
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
check_stmt  := "check" expr "else" "trap" STRING ";"
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
infix_op       := "+" | "+wrap" | "+checked" | "+sat"
                | "-" | "-wrap" | "-checked" | "-sat"
                | "*" | "*wrap" | "*checked" | "*sat"
                | "/" | "/checked" | "%" | "%checked"
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
A `check`, `claim`, or call that may trap also has a normally continuing edge and does not count as delivery or must-divergence.
No `loop_stmt` or `for_stmt` is assumed to diverge.
This recursion is strictly simpler than the ownership checker.
`give e;` moves or copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions exactly as a returned borrow of the same mode [OWN-4].
Only when the enclosing initializer is a `value_if`, its derived delivery mode is `own`, and its type is one [ENT-2] fragment integer may a direct non-consuming bare-atom `give` additionally participate in [ENT-5]'s bounded relation delivery.
The same spelling inside `value_match` carries no relation.
This adds no typing premise and never makes a move, borrow, call, construction, subscript, projection, or computed expression into a fact carrier.
GIVE-1 still owns delivery completeness and exact mode/type agreement; only after those judgments succeed may ENT-5 substitute the atom's already evaluated value into the receiving binding.

For that additional fact-carrier judgment, the direct atom must be one bare tracked own-value binding of the exact receiving type: its root resolves to a body `let_stmt` binding, `for_stmt` binder, parameter, or match binder, and it carries no suffix.
A literal, named const, const-generic constant, Z, counted capture, requires local, projected place, consuming atom, or any other atom may still be delivered as a value but carries no relation through the initializer.
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

The optional `fieldbind_list` in a variant-form `ensures_selector` deliberately keeps zero, one, and multiple written fields parseable.
FN-9, not GRAM-10, owns that selector after its leading TYPEID resolves: it admits exactly the unary `Ok(value: IDENT)` selector of a concrete `Result<T, E>` whose T is one entailment-fragment integer type.
A missing, extra, repeated, misspelled, or out-of-order selector field is therefore an FN-9 rejection at the selector or first offending `fieldbind`, as [DIAG-1] fixes; no match arm or runtime binder is formed.
Every other successfully resolved variant, arity, payload type, nested projection, or nullary selector is outside the postcondition boundary and is rejected by FN-9 rather than generalized through this rule.

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
Call sites state explicitly exactly what their callee class requires: type, region, and const arguments for user generics [FN-2]; region arguments for system operations [SYS-2]; and, for exactly the retained-argument table operations — `cvt` and `reinterpret` (type pairs [OP-6, OP-8]), `array_new` (element type and const length [CONST-1]), `arena_new` (region and element type), `buffer_vacant` (element payload type [OP-1]), and `finf`/`fnan` (result type) — the written arguments their rows fix, because no operand can supply them.
A `construct` of a generic nominal states that nominal's type and const arguments on the same ground and in every position, mandatorily: the source nominals under [FN-2], and the prelude generic nominals `Option<T>` and `Result<T, E>` through their variant constructors `None`, `Some`, `Ok`, and `Err`.
A nullary `None()` has no operand to supply anything, and construction never consults an expected nominal type [TYPE-6], so the written arguments are the only supply there is; their absence, or a count other than the named nominal's parameter list, is a hard error citing TYPE-5 at the complete `construct`.
The non-generic prelude nominals — `Bool`, `Overflow`, `DivError`, `NarrowError` — have no parameters and write nothing.
Every other table operation carries no written argument and derives its selected type from its operands [OP-2]; a written argument there is a hard error citing OP-1.
Argument types match declared parameter types exactly.
After [SET-1] derives a writable target place of type T, the right-hand side of `set p = e;` must produce exactly `own T`; there is no mode coercion, type conversion, or target-selected operation overload.
After the TYPE-7 implicit-read exclusivity below, a different right-hand-side mode or type is a hard error citing TYPE-5 at the complete `expr` child of the `set_stmt`, carrying expected `own T` and the actual mode and type.
After [SET-2] derives a writable affine target place of type T, the right-hand side of `let x = replace p = e;` receives this same exact-`own T` judgment, located at the complete `expr` child of the `replace_let_rhs`.
Redundant-explicit facts remain mandatory at every trust boundary — signatures with full modes, types, effect rows, and regions [FN-1], construction field names [GRAM-8], match binders [GRAM-10], call argument names [GRAM-11] — and are deleted exactly where reconstruction is unique and no transposition risk exists.

Each lower and upper endpoint atom of a `for_stmt` must produce exactly `own u64`; after [TYPE-7]'s implicit-read exclusivity, every other mode or type is a hard error citing TYPE-5 at that endpoint's `atom` node, with `SourceCoordinate` equal to its complete checked half-open source extent.
The counted binder has the fixed compiler-derived mode and type `own u64`; it carries no source annotation and does not infer from either endpoint.

[TYPE-6] Name resolution uses the following closed declaration domains.
The grammar role, never an inferred type or expected result, selects the domain and admissible declaration class.

| domain | declarations | admitted uses |
|---|---|---|
| lexical IDENT | top-level `fn_decl`; top-level `const_decl`; const `gparam`; `param`; `let_stmt`; `for_stmt` binder; arm `fieldbind` binders; one FN-9-admitted symbolic result datum; admitted system operations [SYS-1] | a `callee` IDENT admits a top-level function or an admitted system operation; a `fn_bind` right IDENT admits only a top-level function; `const` IDENT admits only an in-scope const generic or earlier named const; `cvalue` IDENT admits only an earlier named const; `pbase` admits only an in-scope value binding, the one live symbolic result datum, or a named const |
| nominal-type TYPEID | source `struct_decl` and `enum_decl` names; PRE-1 nominal types; admitted system nominal types [SYS-1]; lexical type `gparam`s overlay this domain while live | `type` TYPEID and the TYPEID suffix of a FORM-5 generic numeric literal admit a live type generic where that form requires one, otherwise a nominal type |
| constructor TYPEID | each source struct constructor under its struct TYPEID; every source enum `variant`; PRE-1 variants, classified as struct-constructor or enum-variant; admitted system constructors [SYS-1], classified as struct-constructor or enum-variant | the leading TYPEID of `construct` admits either class; the leading TYPEID of `arm` or a variant-form `ensures_selector` admits only enum-variant |
| contract TYPEID | source `contract_decl` names and PRE-1 contract names, including `Int` and `Float` | the optional bound TYPEID of a type `gparam` and the contract TYPEID of `conform_decl` |
| REGIONID | `region_params` and `region_stmt` | every REGIONID in `type`, `mode`, `targ`, `effect`, and `borrow_expr` |
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
A `fn_decl` parameter becomes visible after its complete `param` through the function's requires block, ensures block, and body.
A `fn_sig` parameter becomes visible after its complete `param` through that signature's terminator; duplicate parameters in that signature are same-scope redeclarations even though there is no lexical value-use role in the remaining suffix.
A `let_stmt` binder becomes visible only after its complete initializer statement through the end of its lexical block; a requires-block let is visible only to later requires entries and not to the ensures block or function body [FN-8], while an ensures-block let is visible only to later ensures entries and not to the function body [FN-9].
A plain `ensures_selector` IDENT, or each second IDENT in its variant-form `fieldbind_list`, is an FN-9-owned result-datum candidate rather than a TYPE-6 declaration event.
After the complete selector it is retained as a provisional candidate record so the resolver need not guess its grammar role.
It participates in FORM-3 reservation checking but not TYPE-6 duplicate or shadow ranks.
FN-9 performs result, selector-shape, owner, field, and freshness admission, including its source-ordered same-spelling ensures-local scan, before resolving any `ensures_entry`; only an admitted candidate becomes the one symbolic result-datum declaration visible inside that ensures block.
It has no runtime storage or ownership state and is not visible in the function body.
A match binder becomes visible in its arm body only after the complete fieldbind list and only after GRAM-10 has established that it differs from its paired field label, every earlier binder in that arm list, and every lexical-IDENT declaration live on arm entry.
A `for_stmt` binder becomes visible only after its complete header, including both endpoint atoms, and only within that counted body.
An ordinary or counted loop label and a local region are visible only in their respective bodies; neither a counted label nor its binder is visible in either endpoint.
A named const becomes visible only after its complete `const_decl`, preserving CONST-2's explicitly-earlier rule.

Within one domain, two declarations in the compilation-unit root or in the same lexical scope are a redeclaration attributed to the later declaration event.
Declarations in unrelated function or declaration owners are not duplicates merely because their spellings match.
A nested lexical declaration may not shadow an entry live at that declaration.
OWN-3's function-wide REGIONID uniqueness is stricter than either rule and is reported at the later region declaration with the conflicting region origin.
GRAM-10 exclusively owns arm match-binder distinctness and freshness: a second IDENT of an arm `fieldbind` equal to its paired field label, an earlier binder in the same arm list, or any lexical-IDENT declaration live on arm entry is rejected citing GRAM-10 at that later/offending binder before it becomes a declaration, rather than also being reported as TYPE-6 shadowing.
FN-9 exclusively owns the analogous result-datum checks: its candidate binder must differ from its paired `value` field and every lexical declaration live at the selector, and no later ensures-local binder may shadow an admitted result datum.
Either failure creates no TYPE-6 declaration or duplicate event.
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
`const-reject` is disjoint from the runtime arithmetic modes: it never overloads a runtime `.trap` OPNAME or a bare infix trap row, an accepted const-expression executes no runtime check and cannot trap, and a const-expression never enters EFF-2's exhibits-traps relation.
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
The bare-affine mechanical fix is position-conditional: outside a `requires` block it is write `move p`, while inside a `requires` block, where [FN-8] rejects `move` itself, it is restate the clause over copy operands or non-consuming admitted reads, so the repair never instructs a spelling FN-8 forbids.
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

A `let` whose ordinary right-hand side is a user call with borrow-mode result is a borrow holder exactly when the callee signature determines one provenance-candidate parameter: the one parameter written as a borrow of the result's kind in the result's formal region, with no other parameter naming that formal region in its mode or written type and with a region-free result type.
resolved(result holder) = the candidate actual's complete resolved place, even when the callee delivered a narrower suffix of it; the holder's borrow is otherwise ordinary — OWN-4 liveness in the substituted result region, OWN-5 exclusivity, OWN-6 child admission, OWN-14 returned reborrow.
Binding a borrow-mode user-call result whose callee signature does not determine a candidate is a hard error citing OWN-6 with the restructuring `give the callee exactly one parameter written as a borrow of the result's mode and region and no other parameter naming that region, or bind the borrow from a direct borrow expression`.
Nothing here narrows FN-1: the caller still judges the call by the signature alone.

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

[OWN-12] Calls (OWN-CALL cluster): at a call, declared region parameters are substituted with the caller's region arguments, which must be live; argument borrows are live accesses of their resolved places for the duration of the call and are judged under OWN-5 (two `&uniq` arguments whose resolved places overlap are an error); the callee's effect row, instantiated at the actual regions, is checked against the caller's live borrows under OWN-5.
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

A compiler-owned resource family additionally fixes exactly one release action in its normative family contract.
That action may perform one host call, and it carries exactly the effect row that contract fixes, which may include `external` and, where the contract permits synchronous waiting, `blocks`.
A release action's fixed row is the sole input to [EFF-2]'s release contribution; a type whose action carries the empty row contributes no release effect anywhere.
No source construct selects, replaces, supplies, suppresses, reorders, duplicates, or observes a release action, and no release action is conditional on a source declaration.

There are no finalizers in the writer-registered sense: no source declaration, annotation, attribute, contract, conformance, or binding attaches a writer-defined action to a value's release, and this specification defines no construct that could.
This clause does not forbid the compiler-owned release action above, which is fixed by the language and its family contracts rather than registered by a writer.

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
Before the allocator is called, generated code checks or otherwise establishes that the complete runtime byte count has an exact value-preserving representation in the target allocator-parameter domain, and the allocator receives exactly that value.
Every emitted target address computation must likewise be valid for every runtime value that reaches it: generated code checks or otherwise establishes that each runtime index and each mathematically scaled byte offset actually used by the computation has an exact value-preserving representation in the applicable target address-index domain, and that scaling and offset addition do not wrap.
An [OP-4] bounds judgment together with an established complete-object-layout or successful-allocation invariant may discharge these obligations; a backend's implicit narrowing does not.
A failed dynamic target-domain guard follows a non-continuing TCB/resource-failure path before allocator invocation or address formation.
It is not a source rejection, [OP-4] bounds trap, new language trap, or [DIAG-3] event.
`buffer_new`'s distinct u64 multiplication-overflow condition remains exactly [OP-9].

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
| `+` `-` `*` | all int T | `(T, T) -> own T` | traps (outside OP-2's constant-operand class) |
| `+checked` `-checked` `*checked` | all int T | `(T, T) -> own Result<T, Overflow>` | pure |
| `/` `%` | all int T | `(T, T) -> own T` | traps |
| `/checked` `%checked` | all int T | `(T, T) -> own Result<T, DivError>` | pure |
| `ineg.wrap` | signed int T | `(T) -> own T` | pure |
| `ineg.trap` | signed int T | `(T) -> own T` | traps |
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
| `buffer_new` | `T` copy (v0: primitive) | `(u64, T) -> own buffer<T>` (allocates a flat buffer of the u64 length and fills every element; T1) | allocates(heap), traps |
| `buffer_vacant` | `T` region-free [STOR-5] | `(u64) -> own buffer<Option<T>>` (allocates a flat buffer of the u64 length; every element is `None()` of `Option<T>`, compiler-minted, no source value duplicated; T1) | allocates(heap), traps |
| `iand` `ior` `ixor` | all int T | `(T, T) -> own T` | pure |
| `inot` | all int T | `(T) -> own T` | pure |
| `ishl.wrap` `ishr.wrap` | all int T | `(T, u32) -> own T` | pure |
| `ishl.trap` `ishr.trap` | all int T | `(T, u32) -> own T` | traps |
| `irotl` `irotr` | all int T | `(T, u32) -> own T` | pure |
| `ipopcount` `iclz` `ictz` | all int T | `(T) -> own u32` | pure |
| `ibswap` | int T, width>=16 | `(T) -> own T` | pure |
| `imulhi` | all int T | `(T, T) -> own T` | pure |
| `+sat` `-sat` `*sat` | all int T | `(T, T) -> own T` | pure |
| `imin` `imax` | all int T | `(T, T) -> own T` | pure |
| `iabs.wrap` | signed int T | `(T) -> own T` | pure |
| `iabs.trap` | signed int T | `(T) -> own T` | traps |
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
Let `ModeWords` be exactly the suffix alternatives in FORM-3's active OPNAME formation rule together with the operator-form suffixes of [GRAM-1]; in this version the two carriers share one closed set, `{wrap, trap, checked, sat, strict}`.
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

No source declaration or FN-9 result-datum candidate in this closed list may use a member of `ReservedLowerNames`: the IDENT of `fn_decl`; the IDENT of `const_decl`; every `param` IDENT; every `let_stmt` IDENT, including ordinary, propagate, value-match, value-if, requires-block, and ensures-block lets; a plain `ensures_selector` IDENT; the second IDENT of any `fieldbind`; every `field` and `vfield` IDENT; and the IDENT-shaped interior of `region_params` and `region_stmt`.
Such a reserved spelling is rejected citing exactly FORM-3 before freshness ownership is considered.
Dependent field declarations participate in this pre-resolution reservation inventory even though their owner/member duplicates remain deferred.
No other declaration role is covered: type-generic TYPEIDs, const-generic IDENTs, LABELs, and contract-member `fn_sig` IDENTs remain outside this prohibition.
Dotted OPNAMEs cannot be declarations under the grammar.
This reservation keeps operation-versus-function resolution context-free [META-2] and keeps a field-access place from maximal-munching as OPNAME [FORM-3].

[OP-2] Exact add, subtract, multiply, negation, and integer-comparison semantics are defined over mathematical integers, never over host-language overflow.
The closed integer-type set is `i8 i16 i32 i64 u8 u16 u32 u64`.
For `iK`, where K is 8, 16, 32, or 64, the value set is `[-2^(K-1), 2^(K-1)-1]`.
For `uK` it is `[0, 2^K-1]`.
Let `M = 2^K`.
For any mathematical integer z, let u be the unique integer satisfying `0 <= u < M` and `u ≡ z (mod M)`.
Define `wrap_uK(z) = u`; define `wrap_iK(z) = u` when `u < 2^(K-1)`, and `wrap_iK(z) = u - M` otherwise.
This definition fixes two's-complement wrapping without depending on a host integer type or a host remainder convention.

For `a +wrap b`, `a -wrap b`, and `a *wrap b` over a common selected type T, let z be respectively the mathematical sum, difference, or product of a and b.
The operation returns `wrap_T(z)`.
These operations are total and pure for all values of every integer T; they never trap and never produce a runtime overflow check.

For `a + b`, `a - b`, and `a * b` over a common selected type T, let z be the same mathematical result.
A bare-operator call at least one of whose two operand atoms reads as an [ENT-2] constant — an integer literal or an integer-typed named const, judged per concrete [FN-2] instance — is in the constant-operand class.
A constant-operand-class call carries the overflow obligation that z belongs to T's value set [ENT-6], judged by the same complete-state base discharge as a subscript bounds obligation.
A discharged class call returns the exact value z with no runtime overflow check in any build mode, never traps, exhibits no `traps` under [EFF-2], and its checked-program disposition records the discharging derivation [DIAG-2].
A class call whose obligation the complete fact state does not discharge is a compile-time rejection citing OP-2 at that call's `infix` node, carrying the residual obligation rendered exactly per [ENT-6]; it publishes no checked program.
Its mechanical fix is a dominating `claim` of the residual [CLM-1], a dominating branch establishing it [ENT-3], or the explicit `wrap`, `checked`, or `sat` respelling.
A class call whose two constant operands make overflow inevitable instantiates a ground false conjunct [ENT-6] and is therefore rejected at every non-contradictory point; there is no accepted always-trapping bare spelling.
For a class call in a [CLM-3] demanded strict component, the same normalized obligation must additionally discharge in that function's already-computed unasserted U state [ENT-6]; a refuted or unproved strict judgment is a hard rejection citing OP-2 at the same `infix` node, carrying the same exact residual plus the strict root, concrete function instance, and `unasserted` view, and its mechanical repair is [OP-4]'s strict repair.
A bare-operator call both of whose operand atoms are non-constant retains the trapping judgment: if z belongs to T's value set the operation returns that exact value, and otherwise it traps for integer overflow before producing a result.
Integer overflow in one of these retained bare-operator trapping operations is a contract violation [ERR-4, SCOPE-4], not a recoverable `Overflow` value, source rejection, wrapped result, saturation, truncation, or undefined behavior.
Each retained trapping call syntactically exhibits `traps` under [EFF-2], even when a proof eliminates its runtime overflow test.

For `ieq(a, b)`, `ine(a, b)`, `ilt(a, b)`, `ile(a, b)`, `igt(a, b)`, and `ige(a, b)`, both operands denote their mathematical values in the selected T.
The result is respectively `True()` exactly when `a=b`, `a≠b`, `a<b`, `a<=b`, `a>b`, or `a>=b`, and is `False()` otherwise.
Ordering on `iK` is signed mathematical ordering; ordering on `uK` is unsigned mathematical ordering.
Equality and inequality compare values of the same exact T; they do not convert widths or signedness.
All six operations are total and pure and produce `own Bool`; they never overflow, trap, or create a runtime check.

Each operation in the preceding paragraphs carries no written type argument: its selected type is derived from its operands.
Both operands must have one identical exact type — a member of the closed integer-type set or, in a symbolic generic body, one live type parameter whose bound resolves to PRE-1 `Int`, with every concrete FN-2 instantiation substituting one closed-set member and the corresponding mathematical semantics above.
That common exact type is the selected type; the derivation is agreement, never widening, conversion, or preference.
Operands of two different exact types are a hard error citing TYPE-5 at the second operand atom in source order.
A written type argument, a region or const argument, a concrete operand type outside the closed integer set, an inadmissible generic type, or a wrong operand count cites [OP-1].
The implicit-read case already owned by [TYPE-7] is exclusive: when an operand uses a borrow-mode or box/arena binding where its referent selected type would be required, that use is rejected citing TYPE-7 and the operation's wrong-type judgment forms no rejection.
Every other operand whose type is not exactly the selected type cites [TYPE-5].
The operation judgment produces the exact table result type.
A mismatch with a consuming construct is owned and located by that construct's numbered rule; OP-2 does not reattribute it.
After applying the same TYPE-7 exclusivity, TYPE-5 owns let-binding and call-argument exactness, OP-5 owns the check condition, and FN-1 owns return exactness.
No operand value or expected result type selects the operation type, changes the operation family, or inserts a conversion.

For `ineg.wrap(a)`, `ineg.trap(a)`, and `ineg.checked(a)`, T is the operand's exact type, one signed member of the closed integer-type set, and z is the mathematical integer `-a`.
`ineg.wrap(a)` returns `wrap_T(z)` and is total and pure; in particular, negating T's minimum value returns that same minimum value.
`ineg.trap(a)` returns z exactly when z belongs to T's value set and otherwise traps for integer overflow before producing a result.
That overflow is a contract violation [ERR-4, SCOPE-4] with the same nonrecoverable judgment as trapping add, subtract, and multiply.
`ineg.checked(a)` returns `Ok(value: z)` exactly when z belongs to T's value set and otherwise returns `Err(error: Overflow())`.
Thus the sole exceptional input for the trap and checked rows is T's minimum value, and the overflow classification is table-fixed [ERR-4].
A constant minimum operand remains a well-typed accepted call; when executed it wraps, traps, or returns `Err` according to the written row.
The trap row syntactically exhibits `traps` under [EFF-2] even when a proof eliminates its runtime check, while the wrap and checked rows are pure.

Each negation call carries no written type argument and has exactly one positional atom operand; that operand's exact type is the selected type.
The type-derivation and symbolic-generic judgments are the same as in the earlier two-operand operation judgment paragraph — the selected type is derived from the operand, never written — except that every concrete selected type must belong to the signed subset `i8 i16 i32 i64`; selecting an unsigned integer or another domain cites [OP-1].
The same OP-1 written-argument, argument-kind, and operand-count judgments, and the same TYPE-7 implicit-read exclusivity, exact table-result, and consuming-construct judgments, apply.
No operand value or expected result type selects the negation mode, changes its signed domain, or inserts a conversion.

There are no wrap modes for division/remainder because no sound modular semantics exists for divisor-zero; this is table data, not an exception clause.
(Negation has a wrap mode: two's-complement wrapping negation is sound modular arithmetic — ledger fix 2026-07-07.) Integer division and remainder have two checkable failures: a zero divisor for all int T, and, for signed T, the single signed-overflow case `iK::MIN / -1` (LLVM sdiv/srem are UB on both); the bare `/` and `%` operators trap on either, and `/checked` and `%checked` return `Err(DivideByZero())` for a zero divisor and `Err(DivOverflow())` for signed minimum divided by negative one, else `Ok`.
DivOverflow is statically unreachable for unsigned T; the uniform `DivError` type is retained for regularity.
Both classifications are table-fixed [ERR-4].
Mode-axis membership per family is table data: add/sub/mul carry {wrap, trap, checked, sat}; div/rem carry {trap, checked}; ineg and iabs carry {wrap, trap, checked}; shifts carry {wrap, trap}.
Masking a shift amount discards writer intent, so a trap rung is offered; masking a rotate amount is the exact identity, so rotates are dotless-total [OP-8].

[OP-3] Float ops that ROUND carry `.strict` (IEEE 754, no reassociation, no contraction) and are the family a future fast-math mode would relax: `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` `fsqrt.strict` `ffma.strict`.
Float ops that are EXACT or exact-selection are dotless: `fneg` `fabs` `fcopysign` `fmin` `fmax` `ffloor` `fceil` `ftrunc` `froundeven` `frem` and the six comparisons.
Approximation/fast-math modes remain an OPEN numeric-semantics question; a relaxed float op would be introduced as a distinct OPNAME (FORM-1-additive).

[OP-4] A subscript `p[i]` selects one element place of an indexable base: the base place `p`'s final selected type must be `array<T, N>`, `slice<'r, T>`, or `buffer<T>`, and the subscripted place's selected type is exactly that element type T — derived from the base place's already-fixed type [TYPE-5] — written where the binding carries an annotation, derived at a body `let` — by the same declared-type selection that types a field suffix, never from expected type or cross-statement inference; a subscript whose base's final selected type is not one of the three indexable types is a hard error citing OP-4 at that subscript's `psuffix` node.
The subscript carries the bounds obligation `i < len(p)` [ENT-6].
A discharged subscript reads or writes with no runtime bounds check in every build mode, and its checked-program disposition records the discharging derivation [DIAG-2].
Base discharge is judged before provenance: a subscript whose obligation the complete fact state does not discharge is a compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying the residual obligation rendered exactly per [ENT-6]; it forms no [PRV-2] or [PRV-3] candidate and publishes no checked program.
Its mechanical fix is a dominating `claim` of the residual [CLM-1] or a dominating branch establishing it [ENT-3].
Only after complete-state discharge succeeds may the constrained-subject gate replace that success with a [PRV-3] local-leaf rejection or retain a downstream demand for [PRV-2].
Discharge is a deterministic checker derivation [ENT-1]; a solver result never participates.
A `buffer<T>` obligation is over the runtime length term.
The offset atom has exact value mode and type `own u64`; after the [TYPE-7] implicit-read exclusivity, any other offset mode or type is a hard error citing OP-4 at the offset `atom` node, with `SourceCoordinate` equal to that atom's complete checked half-open source extent.
A subscript in a [SET-1] target forms the selected place without reading its stored value; its base and offset are evaluated during target evaluation, and its discharge judgment is identical in target position.
A successful bounds judgment neither narrows nor authorizes narrowing the offset or its scaled byte offset; target address formation additionally obeys [STOR-6].
The range validation of the system transfer operations [SYS-8] is an operation-internal contract check with table-fixed trap semantics [ERR-4] whose trap record uses the operation `call` node [DIAG-3]; the discharge judgment does not apply to it.

For a protected subscript in a [CLM-3] demanded strict component, the complete-state base judgment and every applicable [PRV-2] or [PRV-3] judgment above still run first.
After those succeed, the same normalized obligation must additionally discharge in that function's already-computed unasserted U state [ENT-6].
A refuted or unproved strict judgment is a hard rejection citing OP-4 at the same `psuffix` node, carrying the same exact residual plus the strict root, concrete function instance, and `unasserted` view; it creates no new runtime bounds check, provenance event, fallback, fact source, or caller-side duplicate.
Its mechanical repair is a dominating real branch or another non-assertion fact source admitted by [ENT-3]; a body `check` or `claim` is not a strict repair.
An unmarked function outside every demanded strict closure keeps exactly the preceding ordinary judgment.

[OP-5] `check e else trap "msg";` requires `e` to have exact value mode and type `own Bool`, where `Bool` is the PRE-1 nominal type.
No integer, other enum, borrowed `Bool`, or implicit truthiness conversion is admitted [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent `Bool` value would be required, that use is rejected citing TYPE-7 and OP-5 forms no candidate.
Every other exact-mode or exact-type failure is a hard error citing OP-5 at the selected `expr` node, with `SourceCoordinate` equal to that expression node's complete checked half-open source extent.
A conforming check in a function body is a runtime check in all build modes and is never elided.
If `e` is `False()` it emits the required trap record and aborts [SCOPE-4, EFF-4]; if `e` is `True()` execution continues and the checked fact is available only on that dominated continuation.
The final `check_stmt` in a `requires` block uses this exact condition judgment, decoded message, and dynamic-boundary failure behavior, but [FN-8] owns its execution: it is no ordinary-callee runtime check, and only program start plus a later implemented gated adapter evaluate it.
The final `check_stmt` in an `ensures` block uses the exact condition judgment but [FN-9] owns it as a proof obligation; it never executes and has no dynamic-boundary failure behavior.
The fuller stated-and-checked vocabulary (loop invariants, ranges) is DEFERRED with its delta.

[OP-6] cvt partition and semantics (cross-reference TYPE-4).
`cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value is exactly representable in Dst (y the unique such Dst value) and `Err(NarrowError())` otherwise, and it never rounds, truncates, or saturates.
A non-integral float-to-int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or infinity targeting an integer all yield `Err`; for float-to-float, an infinity maps to the same infinity and NaN maps to the target canonical quiet NaN (value-preserving).
A pair is TOTAL — signature `(Src) -> own Dst`, no Result — where every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`.
Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.

[OP-7] Operation-name convention (regularity, W1-predictable).
An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix.
`Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family.
A respelled operation's operator token is its one constant spelling — bare operators carry the trapping-overflow mode and suffixed operators carry `wrap`, `checked`, and `sat` — under exactly the same one-spelling-per-operation discipline; the `i`-prefix convention continues to govern the operations that keep named spellings, the six integer comparisons included.
A `.mode` suffix appears iff the op sits on a mode axis, and single-behavior ops are dotless; the mode axes are the integer result-overflow axis {wrap, trap, checked, sat}, the shift out-of-range-amount axis {wrap, trap}, and the float rounding axis {strict}, with per-family membership fixed by [OP-2].
Signedness-parametric lowering keyed on the operand-derived selected type [OP-2] (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `ilt` = `slt`/`ult` row, not overloading.
Nominal enum identity is likewise checked from the operand-derived selected type before `eeq`/`ene` lowering; equal representation width never makes distinct enum types interchangeable.

[OP-8] Edge semantics and confirmed lowerings for the operations added in this revision; every totality edge is closed here as table data, so no added row is writer-reachable poison (per T2 and W3).
`iand`/`ior`/`ixor` lower to `and`/`or`/`xor` and `inot` to `xor x, -1` (total).
A shift or rotate amount is `u32`; `ishl.wrap`/`ishr.wrap` mask the amount to `amt & (width-1)` and are total, `ishl.trap`/`ishr.trap` trap when `amt >= width`, `ishr` is `ashr` for signed T and `lshr` for unsigned T, and `irotl`/`irotr` lower to `llvm.fshl`/`llvm.fshr` whose amount is taken modulo width, so rotates are total.
`ipopcount` is `llvm.ctpop`; `iclz`/`ictz` are `llvm.ctlz`/`llvm.cttz` with is-zero-poison false, so a zero input returns the bit width (the zero-input fix); counts return `u32`.
`ibswap` is `llvm.bswap` (width a multiple of 16).
`imulhi` is the high half of the full double-width product.
`+sat`/`-sat` are `llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat` clamping to T's range; `*sat` widens, multiplies, and clamps, which avoids the signed-saturation miscompile in `llvm.smul.fix.sat`.
`imin`/`imax` are `llvm.smin`/`umin` or `smax`/`umax`.
`iabs.wrap`/`.trap`/`.checked` use `llvm.abs` with is-int-min-poison false, so `abs(iK::MIN)` is `iK::MIN` (the defined two's-complement edge value): `.wrap` returns it, `.trap` traps on it, and `.checked` returns `Err(Overflow())`.
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

[OP-9] `buffer_new(n, v)` computes its allocation byte-size as the u64 product of n and sizeof(T) (sizeof(T) is a monomorphization-time constant); `buffer_vacant<T>(n)` computes the same product over sizeof(`Option<T>`) and shares every judgment of this rule.
When this product overflows u64, the operation traps [SCOPE-4] before allocating: an unrepresentable buffer size is a contract violation, never a silent under-allocation (R4: no silent corruption; T2: no-UB), so both rows' effect rows include `traps`.
This u64 multiplication overflow is the sole language-level allocation-size trap that `box_new`/`arena_new` (single-T, no runtime multiply) do not have.
After the u64 product succeeds, [STOR-6] separately requires the byte count to have an exact value-preserving representation in every applicable selected-target allocator and address-index domain; failure of that dynamic target-domain guard follows the non-continuing TCB/resource path and is not this OP-9 trap.
Allocation failure (OOM) is handled as by `box_new` (TCB-level, SCOPE-3), not a language trap.
`array<T, N>` performs no runtime size computation: N is a constant-expression fixed at monomorphization, and concrete target representability is checked under [STOR-6].
The language defines no numeric frame limit and no array-size rejection on that basis.
`array_new` remains `pure`: target-layout or resource failure is not program execution, and TCB-level stack exhaustion is not a language trap.

## 8. Functions, generics, contracts

[FN-1] A concrete function's callable boundary states everything ordinary callers need: parameter modes and types, return mode and type, effect row, region parameters, the optional [FN-8] requirement GoalTemplate, the optional verified [FN-9] normal-result RelationTemplate with its complete/unasserted/S4-blinded dispositions, and the derived [PRV-2] provenance column.
The written templates are checked interface claims rather than trusted declarations; a caller consults only their verified finite summaries and never a callee body.
The provenance column is derived from the checked body and closed-unit fixed point, never written.
Adding a protected parameter datum or payload projection to that column is a caller-visible interface change, exactly as strengthening the requirement GoalTemplate or RelationTemplate is.
A generic function carries the same boundary with its written type and const parameters, and each concrete [FN-2] instance substitutes them before its calls and body are re-checked.
A `fn_sig` has neither template.
Function-signature visibility is the [TYPE-6] table.
Every explicit `return e;` must produce exactly the enclosing function's written `rtype`; there is no result-mode or result-type conversion [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent value would be required by the written `rtype`, that use is rejected citing TYPE-7 and FN-1 forms no candidate.
Every other return mode or type mismatch is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child.
FN-9 adds a stricter result and return-expression shape only for a function that elects to declare a postcondition; a function without `ensures_block` retains every return form admitted here.

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

The signature-formation parts of these two slice-result judgments apply equally to a top-level `fn_decl` and a contract-member `fn_sig`: an `own slice` member has the same parameter-derived ceiling, and a borrow-mode direct-slice member is rejected at that member's complete `rtype`.
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
An ordinary `let`, a `let` selecting `replace_let_rhs`, `set`, an expression statement, and a passed `check` or `claim` have a normal edge to `normal_successor(s)`.
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

The optional `deny_claims` terminal is one caller-visible compile-time policy on each concrete function boundary.
It changes no parameter mode or type, return, effect, region, requirement, postcondition, provenance column, runtime body, or lowering, and it is absent from `fn_sig` and [FN-3] signature equality.
One concrete body may serve both ordinary and strict callers.
The derived strict summary reuses the finite concrete ordinary-call graph and SCC condensation already required by [FN-9], retains no foreign function-local derivation identity, and never creates a second graph or body.
A call consults this policy only where [FN-8] and [CLM-3] require the existing U judgment.

[FN-2] Function and nominal generics are monomorphization-only; instantiation arguments are always explicit; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.
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

The bound function has no `generics` child, `requires` block, or `ensures` block.
Region parameters are permitted and are not a `generics` child.
Its callable signature equals the member signature exactly: the two signatures have the same number of region parameters and value parameters; corresponding parameter modes and types, return mode and type, and normalized effect rows are equal after replacing every occurrence of the member's first, second, and later declared region parameters with the bound function's region parameters at those same zero-based ordinals.
This replacement applies inside modes, types, and effect payloads; type components then use the preceding exact concrete-type identity recursively.
After each signature's independently applicable EFF-1 judgment and the bound function declaration's EFF-2 judgment succeed, an effect row normalizes to six capabilities: the set of declared read regions, the set of declared write regions, the allocation set whose members are `heap` and each `arena` region, the presence or absence of `external`, the presence or absence of `blocks`, and the presence or absence of `traps`; `pure` is six empty capabilities.
Region entries use their alpha-mapped declaration identities.
Equality requires all six capabilities to be equal.
`external` and `blocks` are compared by presence exactly as `traps` is, and a `fn_sig` member may declare either.
A `fn_sig` has no body and no compiler-derived release, so it declares these categories without an EFF-2 judgment of its own; the bound `fn_decl` must exhibit exactly the member's declared row under [EFF-2], including a category the bound function contributes only through release.
A member declaring neither category therefore cannot bind a function that exhibits one, and a `pure` member cannot bind an externally effectful function.
Source occurrence order and repeated occurrences do not affect this equality, but no capability may be omitted or added; there is no effect subtyping or semantic implication.
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
The bound function is nongeneric and has neither `requires` nor `ensures` block.
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
That declaration is the unit's entry; it is nongeneric and declares no region parameters.
The entry takes exactly one of the two forms fixed below, and no other declaration in the unit carries a `program_kind` child or an `input_label` child.

The unlabelled entry carries no `program_kind` child, declares no value parameters, and has written result `own unit`.
Its written effect row is exactly one of `pure`, `allocates(heap)`, `traps`, or `allocates(heap), traps`.
This form is complete and unchanged: a unit that requests no standard input declares no program kind and keeps it.

A kind-declaring entry carries one `program_kind` child and declares its standard inputs as labelled value parameters, each written `input_label IDENT ":" mode type` [GRAM-2].
Its written result and its admitted effect categories are fixed by its kind row below.
A kind-declaring entry is invoked exactly once, by program start [PROG-3]; a `call` whose callee resolves to it is a hard error citing FN-7, because its standard inputs are supplied at start and are neither constructible nor forgeable by source.
The unlabelled entry retains its ordinary callee status [TYPE-6, OP-1].

The closed program-kind table is:

| kind | written result | admitted effect categories |
|---|---|---|
| `command` | `own ExitStatus` | `allocates(heap)`, `external`, `blocks`, `traps` |
| `service` | reserved spelling; no form defined | none |
| `embedded` | reserved spelling; no form defined | none |

A `command` entry's written effect row is any subset of that row's admitted categories written in the [EFF-1] canonical order; `pure` is the empty subset.
No region-bearing effect is admitted in either entry form.
`service` and `embedded` are reserved spellings only: this version defines no entry form, lifecycle, standard input, operation, or target qualification for either, and a `program_kind` naming one of them is a hard error citing FN-7.
A `program_kind` naming no row of this table is the same hard error.
As with an [FN-4] law name, the grammar accepts an IDENT here so that syntax formation encodes no semantic vocabulary, and the checker requires that IDENT to equal exactly one table row.
The IDENT is a checked table fact: it declares nothing, resolves to no declaration, enters and queries no name domain [TYPE-6], and reserves no lexical spelling.

The closed standard-input table for kind `command` is:

| ordinal | label | written mode and type | supplied value |
|---|---|---|---|
| 0 | `command.args` | `own Args` | the immutable invocation-argument snapshot |
| 1 | `command.cwd` | `own DirectoryRead` | the read capability for the initial working directory |
| 2 | `command.stdout` | `own Output` | the standard output sink |
| 3 | `command.stderr` | `own Output` | the standard error sink |

Every value parameter of a `command` entry carries an `input_label` and selects one row of that table.
The label's first IDENT equals the entry's `program_kind` IDENT, its second IDENT equals that row's label tail, and the parameter's written mode and type equal that row exactly; there is no conversion, default, or inferred mode [TYPE-4, TYPE-5].
Every row is optional and each may be selected at most once: an unused standard input is omitted, and the selected parameters appear in strictly increasing table-ordinal order, so declared order is the one legal byte sequence [FORM-1, GRAM-8].
A `command` entry that selects no row is admitted and receives no standard input.
The binder IDENT written after `as` is chosen by the writer and is an ordinary `param` declaration in the lexical IDENT domain [TYPE-6].
Ordinal identity, never type identity, selects the supplied value: `command.stdout` and `command.stderr` share one type and remain two distinct inputs.
An unknown, repeated, out-of-order, or foreign-prefix label, a mode or type differing from its row, an unlabelled value parameter of a kind-declaring entry, an `input_label` on a parameter of any other `fn_decl`, and an `input_label` in a `fn_sig` are each a hard error citing FN-7.
No label tail in this or a later kind table is `wrap`, `trap`, `checked`, `sat`, or `strict`: [GRAM-1] forms a kind IDENT, `.`, and one of those five spellings as one operation-name token, so those five tails are unavailable to a label table.

A compilation unit is kind-declaring exactly when at least one of its top-level `fn_decl` nodes carries a `program_kind` child.
That judgment is syntactic and total: it is fixed after grammar derivation and before declaration inventory [DIAG-1], and it does not depend on the kind IDENT naming an admitted row, on the declaring function's name, on any standard input, or on any resolved type.
A unit that is not kind-declaring admits no system declaration and therefore sees no system type, constructor, or operation name [SYS-3].
The declared program kind is the whole of that trigger; the entry's input types are no part of it.

The one canonical byte sequence for a complete four-input command entry header is `command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {`. [FORM-2] renders it without amendment: `program_kind` and `input_label` are neither line-bearing nor block-bearing, so neither introduces a formatting boundary, and the existing attachment sets join `command`, `.`, and the label tail with no bytes while separating `as` and its binder by exactly one space.

The entry states a program's complete standard-input access in its own signature, so no system value reaches another function except as a written parameter [FN-1]: there is no ambient authority, and no entry-supplied aggregate that source can own, name, or pass.
There is no global state and no `'static` region in v0: ambient mutable globals would (a) erode the noalias fact base every function otherwise gets from parameter-only reachability (P0; carding backlog: GlobalsAA-class evidence), (b) create hidden inter-function channels invisible in signatures (W3, FN-1 signatures-as-trust-unit), and (c) pre-seed shared state for the future concurrency layer (T1).
Immutable `const` items [CONST-2] are permitted and are not global mutable state: being read-only they never erode the noalias fact base (reads of frozen rodata add no aliasing hazard), create no hidden inter-function channel (the value is source-determined in the closed unit), and are Shareable-by-construction [CAP-1]; no `'static` region is introduced (borrows of const-rooted places obey the OWN-10 const clause), and there remains no writer-mutable global and no `static mut` analog.
A standard input is not global state: it is one written parameter of one function, owned and moved under the ordinary rules.

A missing entry is an FN-7 rejection at `BundleRoot` [DIAG-1].
A duplicate `main` spelling remains the later-source [TYPE-6] duplicate.
Every other FN-7 rejection uses `SourceNode` with a `SourceCoordinate` equal to the complete checked half-open source extent of the node named here: a reserved or unadmitted kind name, or a `program_kind` on a declaration that is not the entry, at that `program_kind` node; an unknown, repeated, out-of-order, or foreign-prefix label, or an `input_label` outside a kind-declaring entry's parameters, at that `input_label` node; a written mode or type differing from its table row, and an unlabelled value parameter of a kind-declaring entry, at that complete `param` node; a written result differing from the form's fixed result, at that `rtype` node; an inadmissible effect row, at that `effects` node; a generic or region-parameter-bearing entry, at its `generics` or `region_params` child; a call to a kind-declaring entry, at that `call` node; and every remaining entry-form violation at the `fn_decl` node.

The optional [CLM-3] marker may prefix either entry form and is not a `program_kind` child: `deny_claims fn main(...)` marks the unlabelled form and `deny_claims command fn main(...)` marks the command form.
It creates no third entry form, standard input, or kind-declaring trigger; the unit remains kind-declaring exactly when a `program_kind` child is present.
The marked four-input command header is the preceding canonical header with exactly `deny_claims ` prefixed, and the unmarked header remains unchanged.
Each marked concrete entry is one strict root while all ordinary FN-7 entry, callability, effect, result, label, and start judgments remain in force.

[FN-8] Any source `fn_decl`, generic or nongeneric, may carry one `requires` block after its effect row; the fixed grammar terminal `requires` is ineligible for IDENT under [FORM-3].
The grammar deliberately admits every `doc` or `stmt` as the selected child of a direct `requires_entry`; syntax formation does not encode the block's semantic subset.
Before recursively checking any entry, an early FN-8 structural pass requires those selected children to form zero or more `let_stmt` nodes whose selected right-hand side is `ordinary_let_rhs`, followed by exactly one final `check_stmt`, and nothing else.
The pass examines direct entries from left to right: every entry before the final position must select an admitted ordinary let, and the final entry must select a check.
The first entry that violates that shape is reported; an empty block or an all-let sequence instead reports the `requires_block` node for its missing final check.
Thus a nonfinal or repeated check, a `doc`, a `propagate_let_rhs`, a `value_match`, a `value_if`, or any other direct statement shape is a hard error citing FN-8 before any child semantic error can win.

The complete admitted declaration surface is unchanged apart from admitting it on a generic declaration.
Its scope initially contains only the function parameters, named consts, and the function's type and const parameters.
Each let introduces a fresh clause-local own copy value visible to later clause statements, and clause locals are not visible in the body.
Every computation in the block must be an ANF [GRAM-9] call to, or infix spelling of, a non-trapping, total operation-table row with effect `pure`; the final check condition is either a Bool clause atom or one such operation returning Bool.
User-function calls, construction, `move`, borrowing, subscripting, mutation, control flow, allocation, and any trapping operation are rejected citing FN-8; a place is legal only as a non-consuming operand of an admitted table operation (for example `len(deref(out))`).
Normal typing, ownership, the clause-local copy restriction, and no-shadowing rules still apply after the structural pass succeeds.
`requires` remains absent from `fn_sig` and cannot discharge a law under [FN-4]; contract/refinement support is DEFERRED with a recorded delta.

After those judgments succeed, recursively replace every clause-local use by its unique defining right-hand side until no clause local remains.
The resulting finite typed expression, whose result is exact `own Bool`, is the function's one GoalTemplate.
A template datum naming a parameter is identified by that parameter's zero-based declaration ordinal followed by its written field and `deref` projections before call substitution.
A named-const datum retains its declaration identity and projections; a literal retains its exact type and mathematical or nominal value.
Every operation node retains the selected operation-table row, written type and const arguments actually present at that node after [FN-2] substitution, result type, and written operand order.
Clause-local spellings, clause-local NodePaths, and whether identical subexpressions were shared through one let are absent after expansion.
The callee-instance identity and final-check NodePath identify the requirement occurrence for diagnostics and checked metadata but are not part of predicate equality.

Two instantiated goals are identical exactly when these finite typed expression trees are identical.
No equality step commutes operands, folds a named const or literal, reassociates, inverts a comparison, applies De Morgan, eliminates double negation, or otherwise rewrites an operation tree.
In particular a complete `band`, `bor`, `bxor`, or `bnot` tree is one goal that no evidence for its children ever composes: discharging the whole requires the exact whole tree, while an established whole additionally establishes exactly its [ENT-3] signed decomposition set.
When the complete root is exactly one [ENT-3] comparison relation over admitted [ENT-2] terms or constants, [ENT-4] may additionally derive that one goal through its exact L0 projection; a Boolean subtree projects only as an established member of a signed decomposition set, never toward its parent.

At an ordinary source call, the checker first completes callee resolution, concrete generic instantiation, named-argument and exact-type checking, borrow feasibility, and every obligation belonging to an actual expression.
It then substitutes each formal datum in the concrete GoalTemplate with that actual's pre-transfer value image; for a borrow formal this is the resolved referent place and projections, and for an own actual it is the value before any consuming transfer.
A typed literal, named const datum, or resolved place having only field and `deref` projections retains that ordinary datum.
If a referenced own value or resolved borrow referent instead contains a subscript and therefore has no ordinary goal datum, its image is one compiler-owned ephemeral actual-value datum identified exactly by `(concrete caller instance, call NodePath, zero-based argument ordinal, exact checked type)`; any remaining formal field or `deref` projections stay ordered above that datum.
It denotes the value after the actual and its obligations have been checked but before transfer, is not a source place or source-nameable value, and is equal only to the same tuple.
The resulting instantiated goal takes exactly [ENT-4]'s `discharged`, `refuted`, or `unproved` disposition in the complete fact state entering the call, before any argument consume or borrow commit and before any callee write or other effect kill [ENT-5].
This full-state judgment is first: `refuted` or `unproved` is the [DIAG-1] FN-8 call-site rejection, forms no [PRV-2] target, and publishes no checked program.
Only `discharged` reaches [PRV-2]'s call-argument gate, which is judged at the same pre-transfer point from the retained direct demands and exact requirement bridges.
Only a call with no PRV-2 event then permits the existing transfer, call, and normal-return order.
An ordinary caller never receives a fallback runtime check, entry branch, or second callee body.
A source call to the unlabelled `main` uses this ordinary judgment; a kind-declaring entry remains uncallable under [FN-7].

The function body is checked with its one complete requirement goal established true as [ENT-3] source S4, together with the members of its signed decomposition set, and with the exact L0 relation of the complete root or of a member only where that root has the projection above.
There is no executable ordinary-callee prologue.
Direct recursion, mutual recursion, forward calls, and every concrete generic instance use the same finite rule: each written call edge must discharge its own instantiated goal, independently of declaration or traversal order.
The S4 axiom authorizes source checking only; it creates no `llvm.assume`, optimizer fact, body clone, or alternate lowering path, and later body kills apply normally.

Program start is the one implemented dynamic boundary and follows [PROG-3].
After ordinary full-state body acceptance, [PRV-3] treats every labelled `command` input as unconditionally external and judges an entry-local protected leaf before lowering.
An entry-local leaf whose constrained subject is unconditionally external and whose unasserted S2/S3-blinded state fails is rejected directly; when such an external leaf is retained behind this entry's own S4 requirement bridge, it must additionally discharge in the S4-blinded state, with either local rejection owned by PRV-3.
An inherited bridge reached through an entry-body call is checked at that call's selected argument and any rejection is instead owned by PRV-2.
Thus neither the compiler-owned wrapper check nor the body's S4 axiom can launder an external protected leaf, while an internal subject keeps complete-state discharge and a real branch in the body may establish the relation for an external subject and pass.
A requirement unrelated to a protected leaf retains the boundary behavior below.
After ordinary input setup, the compiler-owned entry wrapper evaluates the same complete goal once, before transferring any source owner to the body.
A false result has the final `check_stmt`'s [OP-5] trap semantics and invokes the body zero times; a true result transfers every owner once and invokes the body once.
The wrapper evaluates only the non-consuming reads admitted above, retains sole ownership during that evaluation, and neither calls nor materializes an ordinary helper that accepts a source owner.
A later gated foreign callable boundary remains governed by [GATE-1]; this version implements no such entry, FFI stub, export, or foreign adapter.
The requirement is a checked signature obligation rather than an executed declaration occurrence and contributes no source effect [EFF-2].

After every ordinary complete-state FN-8 and provenance judgment succeeds, [CLM-3] additionally judges a call made inside a demanded strict component, and a call from outside the closure directly into a marked strict root, in the caller unasserted U state at the same pre-transfer point and with the same exact concrete substitution.
A requirement-free call is discharged trivially.
At one call, a nonempty imported `MayClaims` set is tested first and is owned only by CLM-3; otherwise a refuted or unproved U requirement is one FN-8 rejection at the existing complete `call` node, with the ordinary payload plus the strict root, caller instance, and `unasserted` view.
It has no fallback check, alternate entry, body clone, or duplicated caller-side strict-summary event.
Its mechanical repair replaces the ordinary assertion options with a dominating real branch or another non-S2/S3 fact source admitted by [ENT-3]; an ephemeral actual may first be bound non-consumingly as ordinary FN-8 already permits, but a body `check` or `claim` is not a strict repair.
For a marked program entry with a requirement, the same concrete goal must discharge in U after ordinary standard-input setup but before the compiler-owned wrapper check, owner transfer, or S4 establishment; failure cites FN-8 at the requirement final `check_stmt`.
Success never removes or replaces the one runtime wrapper evaluation fixed below.

[FN-9] Any source `fn_decl`, generic or nongeneric, may carry one `ensures_block` after its optional [FN-8] `requires_block` and before its body.
The fixed grammar terminal `ensures` is ineligible for IDENT under [FORM-3].
The block declares one verified normal-return relation.
It is neither an executable epilogue nor a trusted assertion, and it is absent from `fn_sig`, contract members, system-operation declarations, and every dynamic-boundary surface.

The grammar deliberately admits every `doc` or `stmt` as a direct `ensures_entry`.
Before recursively checking any entry, an early FN-9 structural pass requires zero or more `let_stmt` nodes whose selected right-hand side is `ordinary_let_rhs`, followed by exactly one final `check_stmt`, and nothing else.
It examines direct entries left to right and reports the first entry that violates that shape; an empty or all-let block instead reports the `ensures_block` for its missing final check.
A nonfinal or repeated check, `doc`, `propagate_let_rhs`, `value_match`, `value_if`, `claim`, user or system call, construction, `move`, borrow, subscript, mutation, control flow, allocation, and every trapping or partial operation are inadmissible.
After the structural pass, every clause computation must be one ANF [GRAM-9] call to, or infix spelling of, a pure, total, non-trapping operation-table row.
Normal typing, clause-local own-copy, FORM-3, and declaration-before-use rules apply.
An ensures local is visible only to later ensures entries; neither it nor the symbolic result datum is visible in the function body.
The final `check` is proposition syntax only: its message, clause-local spelling, and sharing have no identity, it contributes no `traps`, it never executes, and it emits no [DIAG-3] record.

A plain `ensures_selector` IDENT is admitted only when the written result is `own T` and T is one [ENT-2] fragment integer after concrete [FN-2] substitution.
It declares that whole result as the symbolic result datum after the FN-9 freshness judgment succeeds.
A variant selector is admitted only as exact `Ok(value: r)` for written result `own Result<T, E>`, where T is one fragment integer after substitution and r is the sole fresh symbolic result datum; `Ok` and its `value` field retain their PRE-1 declaration identities.
FN-9 owns the selector field list and candidate binder as [GRAM-10, TYPE-6] fix.
After selector-shape and ordinary freshness succeed but before any `ensures_entry` is resolved, scan the structurally admitted direct ensures-local `let_stmt` binders in source order; the first binder whose spelling equals the symbolic result datum is a hard FN-9 rejection, so an accepted ensures local can never shadow that datum.
Borrow-mode, unit, float, struct, array, slice, buffer, box, arena, tag-only, nullary, multi-field, nested-payload, whole-Result, every non-`Ok` variant, and every other nominal result remains legal as an ordinary function result but cannot carry an `ensures_block` in this version.
Omitting `Err` means that an `Err` exit is unselected; it does not assert that `Err` is unreachable.

Recursively alpha-expand the ensures locals by their unique definitions exactly as FN-8 expands requires locals.
The final condition must have exact type `own Bool`, and its complete expanded root must be exactly one of `ieq`, `ine`, `ilt`, `ile`, `igt`, or `ige`.
Both operands must be the symbolic result datum, a parameter datum with only field and `deref` projections, a named const, a typed integer literal, or `len(P)` for an admitted formal place P.
At least one operand must contain the symbolic result datum.
No operation result, clause local, arithmetic expression, subscript, ephemeral actual datum, Boolean connective, nested result projection, or body local becomes a relation term.
The comparison normalizes to exactly one finite L0 RelationTemplate under [ENT-2]; equality is its ordinary two-bound L0 relation but remains one semantic relation occurrence.
Every parameter datum denotes that parameter's function-entry value image.
The template retains parameter ordinals and projections, selector and field declaration identity, named-const identity, typed literals, concrete type and const substitutions, selected comparison row, operand order, and normalized relation.
It excludes binder spelling, let spelling or sharing, message bytes, clause-local NodePaths, and callee-instance identity.
Its occurrence identity is `(concrete function instance, ensures_block NodePath, 0)`.

A plain selector selects every explicit `return`.
For an `Ok` selector, a return is selected only when its complete expression is the direct canonical `Ok<T, E>(value: atom)` construction of the written Result type; the selected payload atom is its result datum.
A direct canonical `Err<T, E>(error: atom)` return and [ERR-3]'s automatic propagated error edge are unselected and publish no relation.
Every other Result-valued return expression in a function carrying an `Ok` selector is unsupported and is a hard FN-9 rejection rather than an inferred tag or payload path.
At a selected return, the whole result or selected payload must evaluate to one [ENT-2] term or constant.
For every concrete instance, the selected-return set must be nonempty.
An empty set is a hard FN-9 rejection at the `ensures_selector` with residual `no selected normal exit`; this explicit non-vacuity surface rule is not implied by FN-8, GIVE-1, or logical quantification and may be relaxed only by a later specification version.

Each parameter datum referenced by the RelationTemplate denotes an entry image but creates no snapshot term.
Its entry-image stability begins live at function-body entry and becomes permanently unavailable on the first structural normal edge carrying an [ENT-5] kill that overlaps the exact datum, any holder used by it, or its ordinary support.
Stability joins by intersection over all structural inputs reaching a selected return; contradiction and a later fact over the same source term never restore it.
The fixed-length support rule remains exact: an element write does not invalidate `len(P)`, while a kill of P's root or a holder used to reach P does.
This metadata is view-independent and creates neither an L0 term nor a fact source.

The checker first completes each selected return expression's ordinary typing, obligations, nested calls, effects, and pre-return kill events.
If any referenced entry image is unavailable, that return's relation is `unproved` before a relation query.
Otherwise it substitutes the evaluated result datum into the RelationTemplate and queries the closed state immediately before return transfer and edge-carried cleanup.
No result move, cleanup, divergence, or later replacement value is treated as proof.
Thus a body may mutate a formal and continue normally, but it cannot prove a relation about that replacement and export the relation over the caller's pre-transfer actual.

The complete state, unasserted state U, and S4-blinded state B each judge the same instantiated relation at every selected return, after the one view-independent stability judgment.
Complete, then U, then B is the fixed per-return view order.
Complete discharge at every selected return is mandatory; the first source-ordered selected return whose complete relation is refuted or unproved is a hard FN-9 rejection with that exact disposition and no runtime fallback.
For each of U and B, retain the ordered per-exit dispositions and mark the aggregate discharged exactly when every selected return discharges in that view; U or B failure does not reject the declaration.
A complete-only summary may therefore depend on a body check or claim, while a U-but-not-B summary may depend on the proved function requirement.
Those distinctions are checked metadata, never writer annotations.

Postcondition verification introduces no summary fixed point.
Form the finite concrete ordinary-call graph after resolution and [FN-2], take its strongly connected components, and process the component DAG with callees before callers.
While verifying one component in all three views, every S12 publication from a call whose callee is in that same component is unavailable; all other ordinary facts and every already verified summary from a strictly earlier callee component remain available.
A recursive or mutually recursive function whose selected exits prove independently may pass, but a proof needing a same-component S12 fact is unproved and a seedless cycle establishes nothing.
After every postcondition-bearing concrete instance in the component has mandatory complete success and its U/B metadata is computed, make all summaries of that component available atomically to later caller components.
Declaration order, worklist order, and iteration cannot change this schedule, and no cyclic derivation root is formed.

For one ordinary call c, let `A0(c)` mean that resolution, concrete type/const instantiation, named-argument and exact-type checking, borrow feasibility, every actual-expression obligation, exact pre-transfer formal/projection substitution, and complete-state FN-8 success have all succeeded in that order.
Every one of those judgments is made in the pre-transfer state.
An FN-8 failure forms no postcondition candidate.
`A0(c)` is deliberately independent of the later PRV verdict; it is not checked-program publication authority.

For one verified relation q, `M(c,q)` holds only at one admitted establishment event: q's selector matches that exact result route, its result and every referenced formal can be substituted independently to [ENT-2] terms or constants at that event, and every resulting support is live after the event's ordinary kills.
A non-ENT-2 actual suppresses only a q that references that actual; it does not suppress a relation at another call or one whose template does not reference that formal.
An FN-8 ephemeral actual-value datum is immediate-goal-only and can never satisfy `M(c,q)`.
A discarded result, nested call expression, stored whole outcome, propagated whole outcome, unsupported selector, unselected arm, or killed substitution makes `M(c,q)` false without rejecting an otherwise valid call.

Let Cq, Uq, and Bq mean that q's respective nonempty selected-exit aggregates are discharged.
Let `Gv(c)` mean that every actual-expression obligation and, when present, the exact instantiated FN-8 requirement discharges in caller view v at the same pre-transfer point; a requirement-free call has a trivially discharged requirement conjunct.
Subject to `A0(c)` and `M(c,q)`, failure-atomic scratch establishes q after ordinary transfer and all applicable consumes, borrow commits, callee-effect kills, and target kills: in complete exactly when Cq; in U first when Bq, otherwise only when `Uq and GU(c)`; and in B first when Bq, otherwise only when `Uq and GB(c)`.
This Bq-first order fixes evidence identity when both alternatives hold.
Every establishment retains A0's complete actual-obligation and FN-8 goal parents.
A Bq branch additionally retains the B aggregate parent and no same-view Gv parent; only the Uq branch additionally retains the U aggregate and exact same-view Gv parents.
A complete-only summary never enters U or B.
A U-but-not-B summary crosses one caller view only through that view's exact call premises.
A B summary is independent of assertions and S4.
No view borrows evidence from another.

The admitted result routes are closed.
A plain fragment result may establish onto the fresh binding of a direct ordinary-let call.
A selected `Ok` payload may establish only when the ordinary call is the direct scrutinee of that `match_stmt` or `value_match`, and only at entry to its exact direct `Ok(value: payload)` arm after dispatch creates the bare payload binder.
`let outcome = call(...);` establishes no pending S12 fact or token; a later match of that named outcome establishes none, regardless of intervening events.
The same negative applies to an aliased, stored, propagated, or otherwise indirect whole outcome.
This direct-versus-named ordering is semantic and source-identity independent.

One narrow plain-result receiver route is additionally admitted.
In `set x = user_call(...);`, x must be a previously live bare own fragment binding; the call must return x's exact type; and exactly one argument must be the direct non-consuming bare own-value atom x.
Map every formal to its pre-call actual image, perform argument transfer, effects, and writes, then the ordinary target commit and kill; only afterward substitute the symbolic result with x's post-write value.
`M(c,q)` is false unless q omits the formal supplied by x and every other referenced actual and holder remains live and is disjoint from x under [OWN-7].
A projected target, consuming or non-bare target actual, repeated or overlapping occurrence, distinct receiver, non-call RHS, relation mentioning the overwritten formal, or non-ENT-2 substitution establishes nothing.
This route creates no equality between pre-call and post-call x and is not general `set` or RHS fact transfer.

One narrow selected-payload receiver event is admitted only after the direct-call arm event above has already established q on its payload binder.
The first arm statement must be exactly `set outer = payload;`, with payload the direct non-consuming bare own fragment binder and outer a previously live bare outer own fragment binding of the exact same type.
Evaluate the RHS, perform the ordinary target commit and kill, then replace only result-payload occurrences in that one established q with outer's post-write value and re-establish it in the same C, U, or B view.
Every non-payload support must remain live.
Outer may not be a call actual, occur in q as a non-result term or support, or overlap any other substituted support.
Unrelated old facts on outer die normally and neither authorize nor block the event.
A projected, computed, or consuming RHS; a nonfirst or additional reaching write; a wrong or unselected binder; a named or pending outcome; a different type; aliasing; missing q; or killed support establishes nothing.
Payload scope exit and the ordinary match join then apply normally.
The event establishes neither `outer = payload` nor any unrelated relation.

All facts eligible under `A0`, `M`, and the view formula are placed together in the corresponding complete/U/B semantic scratch before the existing PRV strata are finalized.
PRV-1 component pairs converge and freeze first; PRV-2/PRV-3 demand, bridge, target, and event sets then converge over that one optimistic fact batch.
If any PRV-2 or PRV-3 rejection event exists, its existing rule owns the diagnostic and semantic checking publishes neither any candidate S12 fact nor any checked program.
If no event exists, define `A(c)` for every admitted call candidate and retain all eligible S12 and delivery facts plus the prospective checked program unchanged in that failure-atomic scratch until [CLM-3] succeeds.
A unit with no marker satisfies that additional gate vacuously.
Any CLM-3, strict OP-4, or strict FN-8 rejection discards the whole unpublished batch; only total strict success permits the single finalization and atomic checked-program publication.
No candidate is individually committed or retracted, no no-event premise participates in candidate formation, and no second flow walk, negative fixed point, or partial success exists.
Postcondition verification, relation identity, establishment order, PRV convergence, and the contents of the candidate batch are otherwise unchanged.

Every successful selected-return proof and caller establishment extends [DIAG-2]'s one derivation DAG.
Postconditions add no runtime operation, hidden check, assume, optimizer license, serialized certificate, portable identity, alternate lowering path, or ABI field.

## 9. Effects (gated on exemplar carding before ratification)

[EFF-1] Row grammar: the `effects` and `effect` productions of the fence below, in exactly this canonical order (reads, writes, allocates, external, blocks, traps).

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" REGIONID+ ")" | "writes" "(" REGIONID+ ")" | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "external" | "blocks" | "traps"
```

A category appears at most once in one row.
`pure` is the unique spelling of the empty row and therefore excludes `external` and `blocks` exactly as it excludes every other category.
Frame residency (STOR-1) is not an allocation by definition.
The two added categories take positions between `allocates` and `traps`, which leaves the pairwise canonical order of the four pre-existing categories unchanged.

A category states what a call may do, never which object it does it to.
`external` states that the call may observe or change state outside ordinary Whitefoot memory, including file contents, cursors, output, host namespaces, clock and random sequences, resource lifetime, and compiler-derived resource release [STOR-3].
`blocks` states that an ordinary call may block its current host thread.
Both are payload-free: neither takes a REGIONID, resource name, family name, or any other argument, and `external(cwd)`, `changes(file)`, and every other resource-parameterized effect spelling is outside this grammar and outside this specification.
A source row consequently carries no resource origin, and no rule derives a disjointness, reordering, or elimination conclusion from a row [EFF-5].

`external` and `blocks` are exact fixed grammar atoms and are therefore ineligible for IDENT under [FORM-3], like every other lowercase word this grammar fixes.
The apostrophe- and at-prefixed lexical classes are untouched: REGIONID `'external` and LABEL `@blocks` remain well-formed spellings.

[EFF-2] A concrete function declaration exhibits the union of exactly two contributions: its body-syntactic contribution and its release contribution.
The body-syntactic contribution is syntactic over the complete function body: it exhibits `traps` iff the body contains any trapping-mode operation — a bare `/` or `%`, a bare `+`, `-`, or `*` outside [OP-2]'s constant-operand class, or a `.trap` OPNAME — `check`, `claim`, or a call to any operation or function whose effect row includes `traps` (even if later proven away); it exhibits reads/writes/allocates per the operation table and borrow modes the body uses; and it exhibits `external` or `blocks` iff the body contains a call to any operation or function whose effect row includes that category.
A bare operator inside a `const` [CONST-1] is const evaluation under `const-reject`, not a trapping-mode operation, and contributes nothing to any effect row.
An optional `requires` block is a checked callable-boundary obligation [FN-8], and an optional `ensures` block is a verified normal-return relation [FN-9]; neither is an executed body occurrence, and neither contributes a read, write, allocation, external, blocking, or trapping category.
The release contribution is defined below and has no syntactic occurrence anywhere in the declaration.
A `for_stmt` endpoint and body contribute their ordinary source occurrences under these same clauses, and its body-exit cleanup contributes under the release rule below.
Its compiler-owned captures, binder initialization, header comparison, and representable hidden update contribute no read, write, allocation, external, blocking, or trapping effect.
Function-body attribution and call-boundary projection are separate judgments.

While one function body is checked, every exhibited read or write is attributed after holder resolution and [OWN-5] slice-view provenance.
An ultimate storage root in an own-mode binding of the current function contributes no region read or write, even when reached through a local borrow or local slice view.
A named const root and `immutable-const` likewise contribute no region read; the storage is permanently read-only [CONST-2].
Caller storage reached through an incoming borrow parameter in formal region `'r` contributes `reads('r)` or, for an admitted access through `&uniq`, `writes('r)`.
A formal-slice origin naming any incoming parameter whose direct type is `slice<'r, T>` contributes `reads('r)` when its viewed storage is read, regardless of whether the descriptor parameter's mode is `own`, `&'d`, or `&uniq 'd`.
The descriptor's mode region `'d` still governs the descriptor borrow, but dereferencing that holder does not replace the viewed data origin with the descriptor place.
The slice descriptor itself is not the viewed storage root, and [SET-1] admits no write through it.
A multi-origin slice access contributes the union from every origin by these same clauses.
Binding, moving, passing, returning, reborrowing, and slicing never replace an ultimate origin with a local region spelling.

At a call boundary, each callee or operation `reads` or `writes` entry first retains its formal region declaration identity.
Before any region-argument substitution, that identity selects each occurrence in a formal parameter's written mode or direct `slice` type and therefore the corresponding actual argument projection.
A mode-region occurrence projects a borrow actual through its resolved descriptor or referent place under [OWN-6].
A direct-slice-type region occurrence projects the actual slice value's complete [OWN-5] origin set; when the actual is a borrow of a slice descriptor, holder resolution first reaches that descriptor value and then uses its underlying slice origins.
If one formal declaration occurs in both the mode and direct slice type, both projections are conservatively included.
Only after these occurrence-selected projections are fixed are origins and the effect entry mapped into the caller.
Distinct formal declarations remain distinct for this selection even when the call supplies the same caller region for both; substituting region spellings first never widens the supplier set.
Current-function own roots and immutable const storage contribute no enclosing region effect; roots supplied by the current function's incoming borrow or slice parameters contribute their formal origin regions.
A later read through an `own slice` call result uses FN-1's signature-derived substituted origin union and therefore exhibits every current-function formal-region read that any permitted returned source would exhibit; the callee body cannot narrow this at the caller.
Thus a callee write through a local child reborrow of incoming `&uniq 'r` storage makes the caller exhibit `writes('r)`, while the same callee write through a child reborrow of current-function owned storage adds no caller region effect.
A local region spelling never appears in an enclosing function effect row merely because it was used to form a borrow, reborrow, slice, or arena value.

The release contribution collects the effects of compiler-derived release.
Under [STOR-3] each type fixes one compiler-derived release action together with that action's effect row.
For the function being checked, the release contribution is the union of the effect rows of every release action that may run on any edge of the conservative structural normal-control graph defined in [FN-1].
A release contributes when it may run on at least one such edge; running on only some paths never weakens it, and no path condition, constant evaluation, discharged law, optimizer fact, or backend reachability judgment removes an edge from that graph.
An owner moved or returned on one `match` arm and released on another therefore contributes its release row to the enclosing function, and so does a release derived on only one arm of any other branch, one `give` edge, one propagation edge, or one loop exit.
On each normal edge every owner has exactly one disposition — moved or returned, consumed by an explicit consuming operation, or released by exactly one compiler-derived release action — so one owner contributes at most one release per edge, and an owner consumed on that edge contributes no release there.
Release actions run only on normal edges; a trap runs none and contributes nothing [EFF-4].
A release derived inside a callee belongs to that callee's row and reaches the caller only through the ordinary call-boundary projection of the callee's declared row; it is never attributed to two functions.

This attribution reads only the release rows [STOR-3] fixes, and it does not retrofit memory reclamation into effect rows.
A `box<T>` drop, a `buffer<T>` drop, an `arena<'r, T>` region release, and the absent drop of a `const` item [CONST-2] each carry the empty release row and therefore contribute nothing to any function's exhibited row; only a resource family whose contract fixes a nonempty release row contributes one.
`external` and `blocks` carry no region payload, so the preceding call-boundary projection applies only to `reads`, `writes`, and `allocates` entries: the two categories transfer by presence and are unaffected by region-argument substitution, occurrence selection, and origin projection.

A [SET-1] commit is one write under this attribution, and a [SET-2] commit is one read and one write of the same target origin.
A shared-holder commit is rejected [OWN-5] and contributes no accepted effect judgment.
Effects exhibited while evaluating the target and right-hand side contribute normally; an accepted target subscript is discharged [OP-4] and contributes no `traps`.
Rows are checked both ways against the exhibited row defined above: undeclared-but-exhibited and declared-but-unexhibited are both errors, and a category contributed only by the release contribution is checked exactly like one written in the body.
A mismatch involving the release contribution has no offending source occurrence, so it is a hard error citing EFF-2 using `SourceNode` at that function's `effects` node, with `SourceCoordinate` equal to that node's complete checked half-open source extent; the diagnostic additionally renders the parameter or binding whose release contributed the category, and the restructuring `declare the release effects of every resource this function may release, or move the owner out`.
When more than one owner establishes that premise, the reported one follows DIAG-1's implementation-defined deterministic traversal.
A function whose body and release contribution are empty may therefore declare `pure` while carrying a requirement.
An explicit body `check` or `claim` still contributes `traps` to that caller.
The retained program-start check [PROG-3] and any future gated adapter check [GATE-1] belong to those dynamic boundaries, not to an ordinary source call or the callee's exhibited row.

Canonically, a nongeneric function whose only parameter is `own ReadFile` and whose complete body is exactly `return unit;` exhibits `external, blocks` and must declare exactly that row.
Its declaration contains no call, no `check`, no `claim`, and no other syntactic effect occurrence, so its complete exhibited row is the release contribution of that parameter's compiler-derived release on the function-return edge.
Declaring `pure` is an undeclared-but-exhibited rejection at that function's `effects` node.
This shape cannot be reduced further: [FN-1] requires the body's normal exit to be unreachable, so a function with an empty body is separately rejected and is not the canonical case.

[EFF-3] `pure` licenses deduplication and reordering of calls with equal arguments.
Elimination of an unused pure call additionally requires a termination proof; v0 provides no termination checker, so unused pure calls are not eliminated.
`pure` excludes traps, `external`, `blocks`, and all reads/writes/allocates; it does not promise termination.
These licenses are unchanged for every row that was `pure` before this version, and no license stated here reaches a row carrying `external` or `blocks` [EFF-5].

[EFF-4] Trap is abort: there is no unwinding and no post-violation language cleanup.
The exact [DIAG-3] trap record is the sole mandatory post-violation language output.

[EFF-5] Sequential external calls retain source program order.
Take two calls in one function whose resolved operation or callee rows each include `external`.
If one precedes the other on a normal control-flow path of the conservative structural graph [FN-1] defines, then in every execution performing both, the earlier call's external effect is performed first.
This holds when the two calls name different resources, different resource families, different owners, or the same owner.
A compiler-derived release action [STOR-3] whose row includes `external` participates on the same terms and occupies the position its normal edge gives it, after the releases that precede it in that edge's reverse declaration order.
A call whose row includes `external` is one such ordered point even when the external work is performed inside its callee; the callee's own external calls are performed within that call site's position in this order.
This ordering is a required property of every conforming lowering, at facts-off and at every optimization level; it is not an optimizer fact and no optimizer fact relaxes it.

The rule orders the external calls that one execution performs.
It is not a global runtime lock and not a total order over the whole program: this specification defines no worker, task, thread, or background-submission construct, and when such a construct is added it orders work across executions under its own rules rather than by widening this one.
Independently owned resources therefore remain the mechanism by which real concurrency is expressed later, and this rule constrains only what a single execution has already sequenced.

No target-side fact proves two external calls independent or reorderable.
A native handle or descriptor value, a separate open, a distinct target table entry, a distinct source spelling, the absence of a recorded alias link, and equal or unequal argument values are all outside the source language and prove nothing here.
Reordering, deduplicating, coalescing, hoisting, sinking, speculating, or eliminating an external call is unlicensed: [EFF-3] licenses those only for `pure`, and `pure` excludes `external`.
A separately approved optional fact family may later license one exact transformation through a verifier binding the exact checked-program instance, target, backend, proposition, and authorized consequence [LEDGER-1]; that family's absence, rejection, or resource failure leaves source acceptance and facts-off lowering unchanged.

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
For an enclosing FN-9 `Ok` selector, that automatic error return is unselected and publishes no normal-result relation.
This is Result propagation, not an exception construct or a region in which an exception may be thrown.
Derivation: R4 (keeps recoverable errors shift-left; manual re-match boilerplate invites silent context loss), W1 (one mechanical pattern), W3 (propagation cannot drop the error).

[ERR-4] Classification: expected environment/input failures are values (`Result`); contract violations trap [SCOPE-4].
An operation's classification is fixed by the operation table, never by call site.

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

If the entry has no [FN-8] requirement, the implementation then invokes its body once.
If it has one, the compiler-owned entry wrapper first evaluates that concrete complete goal exactly once over the supplied pre-transfer parameter images, after setup and before the body.
The wrapper remains the sole owner of every supplied `Args`, `DirectoryRead`, and `Output` value while the admitted pure, total, non-trapping expression performs only its ordinary non-consuming reads; the evaluation owns no source value and has no compiler-derived release.
A true result transfers every declared standard-input owner exactly once to one body invocation, whose entry receives S4.
A false result emits the final `check_stmt`'s exact [OP-5, DIAG-3] trap record, invokes the body zero times, transfers no source owner to it, and follows [EFF-4] without a second cleanup path.
The implementation evaluates the expression directly in the sole entry wrapper: it creates no ordinary or checked-IR helper function that takes any source owner, no duplicate body, and no second external entry.
This rule governs both the unlabelled no-input entry and the `command` entry.
A source call to the unlabelled entry is not program start and instead follows [FN-8]'s ordinary static discharge.

For a marked entry, the [CLM-3, FN-8] source-acceptance judgment additionally evaluates the concrete requirement proposition in the existing U proof state over the post-setup, pre-transfer parameter images before the retained wrapper check contributes S2 or the body receives S4.
This is a static derivation query, not a second runtime evaluation.
A requirement-free entry passes it trivially.
If the query succeeds, the exact wrapper evaluation, false trap, true owner transfer, and one body invocation above remain mandatory and only the successful boundary then justifies S4 inside the body.
If it fails, FN-8 rejects at the existing final requirement `check_stmt` and no program instance starts.
The retained check can therefore execute after authorization but cannot authorize itself; no fabricated call, adapter, helper, or second body participates.

Supplying each declared standard input is a start-time obligation of the selected target.
When the selected target cannot supply one, start fails before the requirement goal or body is invoked: no source statement executes, no owner comes into existence, no language cleanup runs, and no `ExitStatus` is produced.
A start failure is a target or environment failure.
It is not a source-language rejection [DIAG-1], not a trap [SCOPE-4], and never rewrites a source acceptance judgment.

A `command` entry that completes normally returns exactly one `own ExitStatus` [FN-1].
Compiler-derived release for every owner live on that return edge runs before the instance terminates [STOR-3].
The selected target then maps that returned value to the process status exactly.
No other source value, written output, effect, release result, or target condition contributes to that status, and the language defines no second normal status channel.
An entry whose written result is `own unit` produces no `ExitStatus`, and this version fixes no process status for that form.

A trap terminates the instance abnormally [SCOPE-4]: the entry's return edge is not taken, no release action runs, and no `ExitStatus` is produced or mapped.
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
Thus traversal reaches a direct name, a name inside a consuming list arm, or a name after one or more skipped nullable prefixes such as `doc?`, but cannot tunnel through structural choices such as `item`, `stmt`, `expr`, `atom`, `callee`, `pbase`, `targ`, `law_arg`, `requires_entry`, or `atom_list | fieldinit_list`.
If several external predicates qualify under the first sentence, their owners rank by first rule occurrence in this specification.
4.
At the `program` `item*` or `item` entry, any `stmt*` or `stmt` entry, or the `requires_entry*`, `requires_entry`, `ensures_entry*`, or `ensures_entry` entry, an IDENT-headed lookahead accepted by no complete construct row cites [FORM-1] as an unknown construct.
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

After canonical FORM-2 succeeds for every source, semantic diagnostic selection first runs the early FN-8 and FN-9 structural-admission passes over every `requires_block` and `ensures_block`.
Within a block, its owning rule selects the specified first invalid direct entry or the block node for a missing final check.
An invalid direct entry uses `SourceNode` at that `requires_entry` or `ensures_entry` production and a `SourceCoordinate` equal to that production's complete checked half-open source extent.
An empty or all-let block missing its final check uses `SourceNode` at the owning block production and its complete checked extent.
These are existing owner-production extents under DIAG-1; no case fabricates a child node, zero-width boundary, or terminal-only coordinate.
Across both block kinds, the minimum tuple `(source_ordinal, byte_start, byte_end, NodePath)` of the selected locations wins; numeric fields compare ascending and NodePath compares as defined below.
No declaration, selector, or use role inside an inadmissible block is classified or counted.
Only complete unit-wide FN-8 and FN-9 structural admission permits role classification and its exact resource-count derivation; only that permits the [SYS-3] system-admission decision, declaration inventory, and lexical resolution in their existing order.
Within an admitted `ensures_block`, the selector's leading lookup and FN-9 selector-admission subjudgment occur before lexical resolution of any `ensures_entry`; every unrelated block and event retains the ordinary global ordering.
Poison declarations and partial resolution are forbidden.
An early FN-8 or FN-9 rejection outranks every inventory or resolution rejection; inventory still outranks resolution even when the later-stage event has an earlier source coordinate.

A semantic role is owned by the lowest production node whose selected right-hand side directly contains the terminal that carries the role; a role reached only through a referenced child production is owned by that child.
A referenced child production means a child production node, not an external terminal predicate such as `literal`.
A semantic role may occupy a complete name terminal, a complete literal terminal, or the exact TYPEID suffix of a FORM-5 generic numeric literal `0_T` or `1_T`.
The suffix role's spelling excludes `_`, and its coordinate is exactly the suffix byte interval.
One token may carry more than one role: for example, a law argument `0_T` has one deferred law-argument role on the complete literal and one lexical generic-type use on `T`.
A struct TYPEID remains one declaration event producing two domain entries, not two events.

Within one owner node, distinct direct grammar-role carriers are ordered left to right by their complete carrier coordinates; distinct carriers with identical complete coordinates use the closed class order declaration, selector-reservation, lexical-use, deferred-use.
The zero-based carrier index is `role_ordinal`.
`subtoken_ordinal` is zero for a role covering its complete carrier; embedded semantic name roles are numbered from one in byte order.
The only multi-role carrier is X09/U18, where the class tie does not reorder the embedded role: a law-argument `0_T` gives its complete deferred argument `(role_ordinal, 0)` and its embedded generic-type use `(role_ordinal, 1)`.
Every role has exactly one owner, class, role ordinal, and subtoken ordinal.
Every declaration event, FN-9 selector-reservation event, lexical-use event, and deferred-use event has canonical key `(source_ordinal, byte_start, byte_end, NodePath, role_ordinal, subtoken_ordinal)`.
Numeric fields compare ascending.
NodePath compares lexicographically by production-child ordinal, with a proper prefix first.
Role and subtoken ordinals are consulted only after the complete path is equal.
For a complete IDENT, TYPEID, OPNAME, REGIONID, LABEL, or literal role, the coordinate is the complete token interval, including a sigil; only the generic-numeric suffix uses a subtoken coordinate.
The event's `SourceNode` names its owner production.
Traversal order, allocation identity, map order, logical path, and inferred type never participate.

Declaration inventory and FN-9 selector reservation create candidates under this closed rank:

1. a FORM-3 reserved-name violation defined by OP-1's derived set;
2. an OWN-3 repeated REGIONID declaration within one function declaration or contract-member signature, parameters included;
3. a GRAM-10 match-binder freshness violation;
4. a TYPE-6 collision with PRE-1;
5. a TYPE-6 collision with an admitted system declaration [SYS-1];
6. a TYPE-6 compilation-root duplicate or same-lexical-scope redeclaration; and
7. a TYPE-6 nested declaration shadowing a live declaration.

Each declaration or selector-reservation event forms an inventory candidate only for an applicable rank above; an event for which no rank applies forms no candidate.
The stage selects the minimum canonical event key among events with at least one candidate and then the first applicable rank at that event.
A FORM-3 reservation payload is `(spelling, carrier_role, reserved_class, inventory_ordinal)`.
Its `spelling` is the complete declaration or selector-candidate spelling.
A REGIONID payload uses its unsigiled IDENT-shaped interior while the rejection coordinate retains the complete sigiled token.
Its closed carrier roles are function, named-const, parameter, let, for-binder, match-binder, plain-result-selector, variant-result-selector, field, variant-field, region-parameter, and local-region.
`reserved_class` is dotless-operation or mode-word.
A dotless-operation ordinal is the zero-based first occurrence among distinct operation-family spellings, scanning OP-1 rows top to bottom and each `op` cell left to right and skipping every later occurrence of the same spelling; both `cvt` rows therefore name one family and one ordinal.
A mode-word ordinal is the zero-based FORM-3 alternative order `wrap`, `trap`, `checked`, `sat`, `strict`.
Those two reserved sets are disjoint in this version.
An OWN-3 repeated-region payload is `(spelling, conflicting_region_origin)` and points to the later region declaration; OWN-3 precedes GRAM-10 in the rank even though no grammar carrier can be both a region declaration and a match binder.
For the GRAM-10 violation defined by TYPE-6, the payload is `(binder_spelling, paired_field_spelling, optional_earlier_binder_origin, ordered_arm_entry_live_lexical_ident_origins)`.
Earlier binders and arm-entry origins are ordered by declaration-event key.
That binder does not also create a TYPE-6 duplicate or shadow candidate.

A TYPE-6 collision payload is `(spelling, ordered_nonempty_conflicts)`.
Conflict domains use the fixed order lexical-IDENT, nominal-type, constructor, contract, REGIONID, LABEL.
Each conflict contains its domain, declaration class, and `conflicting_origin`; conflicts within one domain use PRE-1 declaration ordinal first, then system declaration ordinal, then source declaration-event key.
A source origin is `(NodePath, SourceCoordinate, role_ordinal, subtoken_ordinal)`; a PRE-1 origin is `(PRE-1, declaration_ordinal)`, where `declaration_ordinal` is the zero-based twenty-four-record preorder fixed by TYPE-6; a system origin is `(System, system_declaration_ordinal)`, where `system_declaration_ordinal` is the zero-based preorder fixed by [SYS-2] and appears only in a system-admitted unit [SYS-3].
A struct event may report both nominal-type and constructor conflicts in that order.
Rank 4 reports only PRE-1 conflicts when the same event also conflicts with an admitted system declaration or with source.
Rank 5 reports only system conflicts when the same event also conflicts with source, and is selected for a colliding declaration event at the compilation root and in a nested scope alike, ahead of ranks 6 and 7 at that event.
A PRE-1 collision and a system collision each point to the source declaration.
Rank 6 points to the later source declaration event.
Rank 7 points to the nested declaration, including one shadowing a source-later but whole-unit-visible function.
Every declaration-inventory rejection uses `SourceNode` at the declaration role and has no expected-terminal set.
An FN-9 selector reservation instead uses `SourceNode` at the plain selector or owning `fieldbind`, a coordinate equal to the candidate IDENT token, and the FORM-3 payload above; it creates no TYPE-6 declaration or duplicate event.

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
| `construct` constructor TYPEID, enum-variant-only `arm` TYPEID, or variant-form `ensures_selector` TYPEID | TYPE-6 |
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
The first IDENT and candidate binder IDENTs of a variant-form `ensures_selector`, and the plain selector's candidate binder, are instead FN-9-owned selector carriers.
A plain candidate IDENT produces one selector-reservation event with `role_ordinal` zero in its `ensures_selector`.
In variant form, only the second IDENT of each `fieldbind` produces a selector-reservation event; its `role_ordinal` is one in that `fieldbind`, after the first IDENT's FN-9 field-owner carrier.
A selector-reservation event uses the canonical key above but is not a declaration, lexical use, dependent declaration, or deferred use.
Candidate binders participate in FORM-3 reservation checking but provide no pbase target before FN-9 admission; they enter no owner/member lookup and no TYPE-6 duplicate or shadow inventory.
After the leading TYPEID's ordinary constructor lookup when present, FN-9 alone admits exactly one result datum, which then supplies the sole selector-owned in-block pbase target without creating a second inventory event.
FN-9 rejects a same-spelling later ensures-local binder before entry resolution, so no admitted clause can create a competing pbase target.
The table-checked carriers are exactly the `program_kind` IDENT and both IDENTs of an `input_label`.
Each produces one record for later [FN-7] table checking; none produces a declaration, lexical-use, dependent-declaration, or deferred-use record, none enters or queries a lexical name domain, and none participates in FORM-3's reservation inventory.
The claim-name carrier is exactly the IDENT of a `claim_stmt` [CLM-1].
It produces one record for CLM-1's per-function uniqueness judgment; it produces no declaration, lexical-use, dependent-declaration, deferred-use, or table-checked record, enters and queries no lexical name domain, and does not participate in FORM-3's reservation inventory.
The lexical generic suffix inside a deferred literal law argument additionally receives its ordinary lexical-use record; this X09/U18 pair is the only same-token overlap and produces two distinct role records.
In an `arm` or variant-form `ensures_selector`, the leading TYPEID first resolves globally to an enum variant.
Later typed checking compares that variant's owner with the scrutinee enum for an arm; a foreign arm variant cites TYPE-6.
FN-9 separately requires the selector's successfully resolved variant and owner to be exactly PRE-1 `Result.Ok`.
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
An unknown, repeated, extra, out-of-order, function-`generics`, `requires`-bearing, `ensures`-bearing, or signature-incompatible binding rejects at the offending `fn_bind` and its complete extent.
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
If one of these `.trap` calls makes a function's written effect row disagree with [EFF-2], the EFF-2 rejection uses `SourceNode` at that function's `effects` node and the complete effects extent.

An [FN-8] ordinary-call requirement judgment begins only after every earlier callee, concrete-instantiation, argument, type, borrow-feasibility, and actual-expression-obligation judgment named by FN-8 succeeds.
An unproved or refuted instantiated goal is one hard rejection citing FN-8 with `SourceNode` at that existing `call` node and `SourceCoordinate` equal to the call node's complete checked half-open source extent.
Its deterministic payload contains the concrete callee instance, the requirement occurrence's final-check NodePath, the complete instantiated typed goal, and exactly one disposition, `unproved` or `refuted`; it does not select a predicate by clause-local spelling.
The required restructuring is `establish the complete callee requirement with one dominating branch, check, or claim before the call`.
When the payload contains an ephemeral actual-value datum, it additionally renders that datum as `argument #N pre-transfer value`, with N the zero-based argument ordinal, and replaces the restructuring with `bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow`.
A concrete generic instance that changes a substituted type, const, or datum changes the payload goal and is judged independently.
This rejection is never replaced with a runtime fallback or reported at the callee declaration.

An [FN-9] selector-admission subjudgment begins only after both clause-block structural passes, FORM-3 selector reservation, the selector's ordinary leading-variant lookup when present, and concrete [FN-2] signature substitution.
Admission through freshness precedes lexical resolution or semantic checking of every `ensures_entry`; the remaining clause, selected-return, and proof judgments begin only after those entries resolve and the surrounding function's ordinary semantic judgments required by the failed premise succeed.
Within one selector, test in this fixed order: written result mode/type; plain-versus-variant selector class; resolved variant owner and identity; each written field from left to right against the variant's declaration-order field list; any missing required field after the written list is exhausted; candidate freshness against the paired field and declarations live at the selector; and then the source-ordered later ensures-local binder scan.
A result, class, owner, variant, or missing-field failure uses `SourceNode` at the complete `ensures_selector`.
The first extra, repeated, misspelled, or out-of-order field uses `SourceNode` at that complete `fieldbind`.
A candidate equal to its paired field or any lexical declaration live at the selector uses `SourceNode` at its plain selector or owning `fieldbind`, with coordinate equal to the candidate IDENT token.
The first same-spelling later ensures-local binder uses `SourceNode` at that complete `let_stmt`, a coordinate equal to its binder IDENT token, and payload `(binder_spelling, selector_origin)`, where `selector_origin` is the selector-reservation event's `(NodePath, SourceCoordinate, role_ordinal, subtoken_ordinal)`.
Those are FN-9 events, not GRAM-10 or TYPE-6 duplicates.
An unresolved leading selector TYPEID remains the earlier TYPE-6 lexical-use rejection and forms no FN-9 candidate.

After selector admission, an inadmissible clause computation uses its existing offending entry or expression node; a final condition that is not one exact output-bearing L0 relation uses the final `check_stmt` condition's `expr`.
A concrete instance with no selected normal exit uses the selector and residual exactly `no selected normal exit`.
An unsupported selected return expression, unavailable entry image, or complete relation failure uses `SourceNode` at that existing `return_stmt` and its complete checked extent.
The deterministic relation payload is `(concrete function instance, postcondition occurrence, selector identity, instantiated normalized relation, disposition)`, with disposition exactly `unproved` or `refuted`; entry-image unavailability fixes `unproved`.
Instances use DIAG-1's stable concrete-instance order, selected returns use NodePath order, and the first complete-view failure wins.
U and B are then computed in that order as metadata and form no rejection event.
A later PRV rejection is owned only by PRV-2 or PRV-3; it does not relocate to the FN-9 declaration or publication event.
No FN-9 failure fabricates an executable epilogue, runtime fallback, optimizer assumption, pending named-outcome fact, or caller-side rejection.
An excluded caller route, including a named or pending outcome, is not itself a rejection: it establishes no S12 fact or metadata, and any later query that needed that absent relation is diagnosed only at that later node by its ordinary owning rule.

The [CLM-3] stage begins only after every ordinary source judgment, including CLM-2 refutation, complete OP-4 and FN-8, FN-9, and PRV-2 or PRV-3, has succeeded.
Validate marked roots in the stable concrete-instance order.
If the root SCC has a direct claim, cite CLM-3 at the first claim in stable member-instance then claim-NodePath order and retain `(strict root, concrete claim owner, claim NodePath, name, predicate, justification, lifecycle disposition)`.
Otherwise cite CLM-3 at the first call in stable caller-instance then call-NodePath order within the root SCC whose strictly outgoing callee component has a nonempty `MayClaims` set, retaining `(strict root, concrete caller, call NodePath, concrete callee, least downstream claim identity)`.
A component summary is silent: a claim reached only below that boundary is reported at the importing call, not duplicated at its declaration.
If the root shares an SCC with that claim, the claim is direct and the claim node wins.
At one importing call, CLM-3 is selected before a strict FN-8 U failure; a non-claim strict OP-4 or FN-8 failure is emitted only at its actual leaf or call, and no caller-side summary failure is fabricated.
A marked program-start U failure is FN-8 at its final requirement `check_stmt`.
All these candidates use existing source nodes and the complete extents stated by their owning rules, obey semantic failure atomicity, and publish no checked program or derived ClaimLedger.

A mechanical fix or restructuring is included exactly where the owning rule requires one.
Every published static diagnostic is deterministic for one compiler executable under the conditions above.
Cross-implementation byte identity is required only where this specification explicitly fixes both selection and encoding; the runtime trap record [DIAG-3] is such a case.

[DIAG-2] Successful semantic checking produces one private checked-program value bound to the exact canonical compilation unit.
It is the only input that may grant lowering authority.

The checked program explicitly represents every source operation and every compiler-derived operation required for execution, including drops, arena releases, monomorphized instances, propagation edges, retained runtime checks, every direct slice value's finite origin set, every `own slice` result's FN-1 formal return-origin ceiling and call-site substitution, and one abstract target-domain representability obligation at every runtime-sized allocation and element-address operation governed by [STOR-6].
It additionally retains every [FN-8] GoalTemplate, its requirement occurrence `(concrete callee instance, final-check NodePath, conjunct ordinal 0)`, every concrete call substitution and discharged-goal derivation, the S4 body-entry axiom, and the one retained program-start goal evaluation when the entry has a requirement.
It also retains the converged [PRV-1] component summaries, every [PRV-2] result, write, direct-demand, and bridge column, the complete/U/B outcomes and successful no-event disposition of each accepted call argument, every [PRV-3] local-leaf disposition, and the post-convergence deterministic predecessor choices.
A rejecting PRV-2 target set or PRV-3 witness exists only in failure-atomic diagnostic scratch and is never published as checked-program or lowering authority.
Target lowering must discharge each target-domain obligation from the selected target plus already-checked layout, allocation, and bounds facts, or materialize its exact non-continuing guard before the governed allocation or address operation.
Every potentially removable implicit source-language check has exactly one disposition: `retained`, or `eliminated` with a deterministic checker derivation or separately verified proof that authorizes that exact elimination.
A source subscript carries no implicit check and no such disposition: an accepted subscript is `discharged` at its `psuffix` node, and the checked program retains its exact [ENT-4] derivation for that node.
An explicit body [OP-5] check and every [CLM-1] claim are always `retained`; the checked program retains each claim's name, predicate, and justification STRING.
The final check inside a `requires` block is not an ordinary-callee check: its condition is represented by the GoalTemplate, and an executable retained check exists only for program start [PROG-3] and a later implemented gated boundary [GATE-1].
The final check inside an `ensures` block is represented only by its verified RelationTemplate, selected-exit judgments, and derivations; it is never an executable retained check.
In facts-off compilation every required runtime check remains `retained`, and all [ENT-1] source-acceptance and call-goal judgments are identical in facts-on and facts-off compilation.
Neither a discharged call goal nor S4 authorizes `llvm.assume`, an optimizer fact, or a second lowering path.
STOR-6 target-domain obligations instead follow the target-stage discharge-or-guard judgment above identically in facts-on and facts-off compilation; an optional optimizer fact supplies no target-layout discharge.

The complete, U, and B analyses of one concrete function extend one function-local derivation DAG and one event stream; a view tag distinguishes their nodes.
This is the same authority that already proves accepted obligations, discharged call goals, and S11 facts.
Every parent precedes its child, every retained node is reachable from a required root, and finalization performs one reachability traversal and one identity remap.
An implementation may choose its private Rust layout, but it may not build a postcondition-only proof graph, merge separately authoritative view ledgers, rerun semantic flow to reconstruct a missing root, or consult another checker.
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
A CLM-3, strict OP-4, or strict FN-8 event instead discards them.
This preserves A0/A atomicity and makes the derivation DAG evidence for the accepted semantic flow rather than a second acceptance path.

For [CLM-3], the successful checked program additionally retains each declaration marker, each concrete strict root, its outgoing SCC membership, each component `DirectClaims` and `MayClaims` set, every source-ordered call occurrence used by the summary, the successful component and root disposition, the marked program-start disposition, and the exact existing U derivation root for every demanded protected obligation and call requirement.
The claim and call graph is formed in private semantic scratch from checked claim occurrences and the ordinary concrete call inventory; the checked-program `ClaimLedger` is derived only after success and is never read back as acceptance authority.
Strict roots extend the same function-local view-tagged derivation DAG and event stream; they are registered before the sole reachability walk and identity remap, import no foreign `DerivationId`, and create no copied graph or second semantic flow.
On any CLM-3, strict OP-4, or strict FN-8 event, all strict metadata, candidate S12 and delivery roots, the prospective checked program, and every checked-program-derived tool projection are discarded together.
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

[DIAG-3] The sole mandatory runtime report is one trap record for a failing checked site.
Its exact UTF-8 bytes are:

```text
{"rule_id":RULE,"message":MESSAGE,"function":FUNCTION,"node_path":[COMPONENTS]}
```

The displayed line excludes its Markdown line ending.
The record bytes are the displayed JSON object followed by exactly one byte `0x0A`; no `0x5C 0x6E` suffix is present.

`RULE`, `MESSAGE`, and `FUNCTION` are JSON strings.
Fields occur in exactly the written order with no extra whitespace or fields.
`COMPONENTS` is the source node's zero-based `NodePath`, written as comma-separated unsigned decimal integers with no leading zeros; the empty path is `[]`.

`rule_id` is the exact numbered rule whose runtime condition failed.
`function` is the exact enclosing source function IDENT.
`node_path` identifies the source production that introduced the failing checked condition: the `check_stmt` for [OP-5], including the final `check_stmt` whose complete goal fails at program start [FN-8, PROG-3]; the `claim_stmt` for [CLM-1]; and the operation `call` — or, for an operation spelled infix, the `infix` node — for a table-operation contract check and for the [SYS-8] range validation judged under [OP-4]'s retained operation-internal semantics.
For an executed bare `+`, `-`, or `*` overflow, `rule_id` is `OP-2`, `message` is `integer overflow`, and `node_path` is the trapping `infix` node; such a record arises only outside [OP-2]'s constant-operand class, because a class call discharges at compile time and executes no overflow test; a bare `/` or `%` contract violation is a table-operation contract check at its `infix` node.

For an explicit [OP-5] body check and for an [FN-8] program-start goal, `rule_id` is `OP-5` and `message` is the final `check_stmt`'s STRING value decoded by [FORM-5].
For a [CLM-1] claim, `rule_id` is `CLM-1` and `message` is the claim's exact IDENT spelling; the justification STRING is compile-time data and does not appear in the record.
For every compiler-generated implicit check without a rule-specific message above, `message` is the empty string.

JSON string encoding is canonical for the complete character set defined here: `"` becomes `\"`, `\` becomes `\\`, LF becomes `\n`, and every other permitted ASCII byte is emitted unchanged.
A final single LF terminates the record.

Identical bound source bytes reaching the same failing checked site therefore produce byte-identical report bytes in every conforming implementation.
Dynamic call-stack attribution, artifact identity, successful-check reports, lifetime reports, check-density reports, and optimizer-development reports are not normative outputs.
An implementation may provide additional developer output only on a separately selected channel that cannot alter, prefix, suffix, or replace the mandatory trap record.

## 13. Capabilities (stub; concurrency layer pending)

[CAP-1] Type-level capability predicates of the Send/Sync class exist in the kernel vocabulary: `Shareable` (safe to share across threads) and `Sendable` (safe to transfer). v0 defines no thread construct, so no kernel type is required to declare them; the predicates reserve the vocabulary the concurrency layer will bind.
Data-race impossibility is D1 law; general race conditions are out of scope (C004 amended scope).

## 14. Gated family (writer-visible stub)

[GATE-1] Editing any declared contract, signature, law bundle, storage contract, or gated-family member is one privileged, gated toolchain operation with one audit trail, outside steady-state writer capability.

This version defines no callable FFI import, export, inbound callback, foreign-thread entry, or generated foreign adapter.
If a later amendment admits a gated foreign call path into a source function carrying an [FN-8] requirement, the compiler-owned adapter must validate the same concrete complete goal exactly once after its boundary argument validation and before any owner transfer or body invocation.
A false result must retain the final `check_stmt`'s [OP-5, DIAG-3] trap semantics and invoke the body zero times; a true result must transfer each admitted owner once and invoke the body once.
No foreign caller assertion, manifest fact, ledger entry, or alternate error protocol may stand in for that evaluation unless a later exact specification amendment defines the replacement semantics.
Until that callable path is specified and implemented it remains unsupported compiler capability [DIAG-1]; a fabricated stub or ordinary helper is not boundary evidence.

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
Its complete membership is the system inventory [SYS-2].
Whether a compilation unit admits it is fixed by [SYS-3].

A system declaration is compiler-owned data of this specification.
It is not a source record, include, import, module, separate compilation, dynamic loading, or source-path lookup [PROG-1].
It has no source record, source node, source coordinate, role, or declaration event, and no source construct declares, redeclares, extends, reopens, or overrides it.

The inventory contributes exactly three declaration classes, each already a member of the closed declaration-class order [DIAG-1].
A system nominal type takes the nominal-type class and is an entry of the nominal-type TYPEID domain [TYPE-6].
A system constructor takes the struct-constructor or enum-variant class fixed for it by [SYS-2] and is an entry of the constructor TYPEID domain [TYPE-6].
A system operation takes the function class and is an entry of the lexical IDENT domain [TYPE-6].
The domain contributes no contract, region, label, const-generic, generic-type, value, or operation-family entry, and introduces no declaration class.

In a system-admitted unit every inventory entry is visible throughout the closed unit, is a compilation-root entry of its domain in every lexical use's candidate universe [DIAG-1], and participates in that domain's whole-unit uniqueness [TYPE-6].
That visibility depends on neither the position of the entry declaration, nor record order [PROG-2], nor any source declaration point.
The owner-local field and parameter records fixed by [SYS-2] are visible only within their owning system declaration and never enter source lookup.

In a system-admitted unit a source declaration whose spelling equals an inventory entry's spelling in the same domain is a collision, rejected under [DIAG-1], at the compilation root and in every nested scope alike.
No source declaration displaces, replaces, overrides, reopens, or shadows an inventory entry, and no inventory entry displaces a source declaration: the unit is rejected and neither declaration resolves.
No use of a colliding spelling is decided by proximity, declaration order, scope depth, or expected type.

A system nominal type, constructor, or operation exists only as an inventory entry [SYS-2].
No source construct becomes one by spelling, signature, parameter shape, result type, effect row, or any other source property, in a system-admitted unit or a system-unadmitted one.

Each inventory entry has one zero-based `system_declaration_ordinal` assigned by the [SYS-2] preorder.
That ordinal is the entry's identity in a diagnostic origin [DIAG-1].

[SYS-2] The system inventory is exactly:

The notation here is normative record notation and is not writable source.

Seven opaque nominal types: `Args`, `HostString`, `RelativePath`, `DirectoryRead`, `ReadFile`, `Output`, and `ExitStatus`.
Each contributes one nominal-type entry and no constructor entry.
An opaque type has no writer-visible field, variant, literal, size, alignment, or representation.
It is a complete written `type` under [GRAM-3] as a bare TYPEID with no `targs`, carries no region and no type parameter, and is therefore region-free under [STOR-5].
It is not const-eligible [CONST-2], is not a `cvt` or `reinterpret` domain [OP-6, OP-8], is not an integer, float, `Bool`, or tag-only enum operand domain [OP-1], and has no equality, ordering, or conversion operation.
Its values are produced only by the operations in this rule and by the command entry's standard input bindings.
Every value of an opaque type is affine under [OWN-1].

Seven enum nominal types with thirty-nine variant constructors:

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
  ReadBytes(count: u64);
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
  Interrupted(code: u32, origin: u8);
  WouldBlock(code: u32, origin: u8);
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
```

Eleven operations, each one complete signature record in the [GRAM-2] `fn_sig` shape:

```
fn args_count['a](args: &'a Args) -> own u64 reads('a);
fn arg_get['a](args: &'a Args, position: own u64) -> own Result<HostString, ArgError> reads('a);
fn host_bytes_len['v](value: &'v HostString) -> own u64 reads('v);
fn host_copy_bytes['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own Result<u64, CopyError> reads('v 'd), writes('d), traps;
fn host_utf8_len['v](value: &'v HostString) -> own Result<u64, Utf8Error> reads('v);
fn host_copy_utf8['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own Result<u64, Utf8CopyError> reads('v 'd), writes('d), traps;
fn relative_path(value: own HostString) -> own Result<RelativePath, PathError> pure;
fn open_read['c, 'p](root: &'c DirectoryRead, path: &'p RelativePath) -> own Result<ReadFile, IoError> reads('c 'p), external, blocks;
fn read_once['f, 'd](file: &uniq 'f ReadFile, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own ReadOutcome reads('f 'd), writes('f 'd), external, blocks, traps;
fn write_once['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, offset: own u64, count: own u64) -> own Result<u64, IoError> reads('o 's), writes('o), external, blocks, traps;
fn exit_status(code: own u8) -> own ExitStatus pure;
```

The inventory is therefore exactly fourteen nominal types, thirty-nine enum-variant constructors, sixty-four variant fields, eleven operations, fourteen operation region parameters, and twenty-five operation value parameters.

Each operation's declared region entries are fixed by its own signature: every borrow parameter of formal region `'r` contributes `reads('r)`, and every `&uniq 'r` parameter through which the operation changes the borrowed value additionally contributes `writes('r)`.
The rows above are exactly that derivation together with each operation's fixed external, blocking, and trapping classification; a system operation's row is declaration data and is never derived from a body, narrowed by a proof, or selected by a call site [ERR-4].
Each operation's result components and writable `&uniq` parameter components additionally carry the following closed [PRV-1] provenance classes as declaration data; these classes do not add to or modify an operation signature or effect row.
An operation whose row contains `external` may observe or change state outside ordinary Whitefoot memory, and one whose row contains `blocks` may block its current host thread [EFF-1].
No system operation allocates.

```wf-prov
| operation | result component class | writable `&uniq` parameter component class |
|---|---|---|
| `args_count` | plain result external | — |
| `arg_get` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_bytes_len` | plain result external | — |
| `host_copy_bytes` | `Ok(value:)` internal; `Err(error:)` external | `destination` external |
| `host_utf8_len` | `Ok(value:)` external; `Err(error:)` external | — |
| `host_copy_utf8` | `Ok(value:)` internal; `Err(error:)` external | `destination` external |
| `relative_path` | `Ok(value:)` external; `Err(error:)` external | — |
| `open_read` | `Ok(value:)` external; `Err(error:)` external | — |
| `read_once` | `ReadBytes(count:)` internal; `ReadFailed(error:)` external; `ReadEnd()` carries no result component | `destination` external; `file` external |
| `write_once` | `Ok(value:)` internal; `Err(error:)` external | `output` external |
| `exit_status` | plain result internal | — |
```

A plain-result cell fixes that result's sole aggregate component.
A named payload cell fixes exactly that direct variant-field projection; a payload-carrying result's aggregate is the join of its projections, while a nullary variant and the control choice of a variant carry no component of their own.
An external class seeds the unconditional-external bit; an internal class seeds no bit.
No unlisted result, projection, parameter, field, or component inherits an external class by association.
The internal success components are exactly the program-bounded transfer counts fixed by [SYS-9]; their environment-chosen position within the program-supplied bound does not make them external.

Every system operation is nongeneric: it declares no type parameter and no const parameter, so no `targ` in a system-operation call is a `type` or a `const`.
A call whose callee resolves to a system operation writes its region arguments as `targs` in declared region-parameter order and its value arguments as a `fieldinit_list` [GRAM-5] whose IDENTs equal the declared parameter names in declared order, under the same discipline [GRAM-11] applies to a user function.
Positional operands are not admitted.
A system operation is not a contract member, is not the right IDENT of an [FN-3] `fn_bind`, and never satisfies [FN-4]'s bound-function premise; a conformance binds only a top-level source function.

The inventory contributes exactly one hundred and sixty-seven declaration records in this preorder: each nominal type in table order; then each constructor in table order, and within one constructor each of its fields in declared order; then each operation in table order, and within one operation each of its region parameters in declared order followed by each of its value parameters in declared order.
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

[SYS-3] Every compilation unit has exactly one system-admission state, fixed by one syntactic predicate over its complete finalized item sequence.

A unit that is kind-declaring [FN-7] is system-admitted: the complete system inventory [SYS-2] enters that unit's declaration inventory as fixed by [SYS-1].

A unit that is not kind-declaring [FN-7] is system-unadmitted: the system domain contributes no entry to that unit, every system spelling is an undeclared name there, and a use of one is decided by the ordinary lexical-use ranks [DIAG-1].
A source declaration in such a unit may use any system spelling; it is then an ordinary declaration of its own kind, it collides with nothing, and every use of that spelling in the unit resolves to it under the ordinary domains [TYPE-6, OP-1].

The predicate reads exactly the presence of that program-kind label.
It consults no entry parameter list, standard input label, written type, mode, region, effect row, or result type; no count of entry declarations and no later [FN-7] whole-unit judgment; no resolved name, inferred type, or lowering outcome; and no use of any system name.
It is therefore decided from the finalized compilation-unit tree alone.
It is total, admits every unit to exactly one state, and creates no candidate, event, or rejection of its own. [DIAG-1] fixes the stage at which it is decided.

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
A lease owns no code-unit storage, several live leases may denote the same backing code units, and its compiler-derived release is a logical consume with no host call and no external effect [STOR-3].
Its backing is the command-lifetime argument snapshot that [QUAL-2] requires of every qualified target.
Because that backing strictly outlives every value derived from it, a lease denotes valid code units however it is bound, moved, matched, returned, passed, or stored, and no source-level rule relates a lease to its backing: a lease is neither a borrow nor a region-bearing type, so [STOR-5] places no restriction on storing one and [OWN-5] provenance does not describe it.
That guarantee is a property of the target, enforced at qualification, and is not a judgment over source.
A producer whose backing is not command-lifetime yields no value of this type: it introduces a distinct owned-backing string resource with its own release action and its own family contract, because storage class is a function of type [STOR-1] and one type carries exactly one release action.
Conversion between the two types is an explicit later operation with its own delta [META-5]; no implicit retype, coercion, or representation change relates them [TYPE-4].
Retention of lease identity in the checked program [DIAG-2] serves auditing and lowering; it is not a source-acceptance judgment and refuses no program.

[PATH-1] A relative path is an opaque value whose code units are admitted by construction from one host string and are never assembled, split, or concatenated as source text.
Construction consumes its input host string on success and on failure.
It succeeds exactly when the complete code-unit sequence contains no NUL code unit and begins with no target-root prefix, where a target-root prefix is a code-unit sequence the selected target resolves against a filesystem root, drive, device, or other namespace root rather than against a supplied directory capability.
The exact target-root prefix set is target data fixed by that target's qualification record [QUAL-1]; a Unix-family leading separator and a Windows-family drive or UNC prefix are members of their targets' sets.
Success retypes the same inline lease [HOST-3] with no allocation, no copy, and no code-unit change; failure yields no path value.
Construction preserves every admitted code unit exactly — including `.` and `..` components and every separator — and performs no normalization, canonicalization, case folding, prefix stripping, or component collapse.
A path component type, an absolute path type, and every operation that decomposes, enumerates, joins, or displays a path are DEFERRED additions with their own deltas [META-5].
The first slice constructs one relative path from one host string and supplies it to a directory-relative operation [PATH-2].

[PATH-2] A directory-read capability names one directory object, and a directory-relative operation resolves a relative path against it through the target's own directory-relative resolution.
The capability bound to the command's working-directory entry input is process-equivalent: resolution follows `.` and `..` components, symbolic links, reparse points, and mount transitions exactly as the surrounding process namespace does, so a resolved object may lie outside the directory that capability names.
That is the complete promise this type makes, and it is not a confinement claim.
An implementation presents no stronger one: a target implements directory-relative resolution with its own directory-relative facility, never by concatenating a prefix onto a path and resolving the result against an ambient working directory, and a target with no directory-relative facility fails qualification for the directory-relative semantic IDs [QUAL-1] rather than emulating them.
A confined directory capability — one guaranteeing that lexical traversal, links, mount transitions, and rename races cannot escape a granted root — is a DEFERRED addition with its own distinct type and contract [META-5]; a value's confinement promise is fixed by its type and never changes at runtime.
Absolute paths, cross-root operations, and target-root prefixes require their own inputs and operations, and a directory-read capability admits none of them [PATH-1].

[QUAL-1] Every system operation has exactly one target-independent semantic ID owned by this specification.
That ID's record binds the operation's signature, complete outcome set, ownership transitions, memory and external effects [EFF-1], compiler-derived cleanup [STOR-3], and required target guarantees [QUAL-2].
The checked program carries only the semantic ID [DIAG-2]: an operation's identity comes from resolution in the system declaration domain, and no source function name or spelling, logical path [PROG-2], project, corpus, test, or signature lookalike ever selects, adds, or removes one.
A separate target-qualification table maps each `(specification version, semantic ID, target, program kind)` to exactly one approved implementation version and one private ABI symbol.
The compiler consults that table after selecting the exact target and ABI [STOR-6] and before emitting any use of the operation.
Compilation stops when the mapping is absent, when the approved implementation is incompatible with the selected target or program kind, or when a required target guarantee is unmet.
That stop is a target-qualification failure under [DIAG-1]; like a target-layout failure it is not a source-language rejection and cites no language rule.
Qualification never narrows a semantic ID to what a target can supply, and no implementation substitutes a different or weaker operation for an unqualified one.
An approved implementation may be replaced only within one semantic identity: a change to any element the record binds is a different semantic ID under a new specification version [META-5] and a compatibility review, never a target-code update.
The table is compiler-internal data; the language defines no registry, negotiation protocol, dynamic loading, or plugin interface [PROG-1].

[QUAL-2] A target qualifies for a semantic ID exactly when it supplies every target guarantee that ID's record requires; when it cannot supply one, it fails qualification for that ID and compilation stops [QUAL-1] rather than admitting the operation under a weaker guarantee.
Two guarantees are stated here because each is a property of the target with nothing in a program to check.
The first is command-lifetime argument backing: a target qualified for the command entry and for argument access supplies immutable backing for every argument code unit that is valid from before entry until the command invocation ends, either as stable native argument backing or as one complete snapshot taken before any Whitefoot code runs.
A target that can supply neither fails qualification for both IDs; a qualified target that cannot establish the backing for one invocation refuses startup before entry rather than entering with backing that does not meet this guarantee.
The second is a lossless host-string code-unit family [HOST-1] for the host-string and path semantic IDs.
Qualification failure and startup refusal both occur before entry [PROG-3], so neither is a source-returned status, a recoverable outcome, or a trap [TRAP-1].

[QUAL-3] For a natively compiled command, selection is static for the whole build: [QUAL-1] fixes the approved implementation of each semantic ID at compile time, and the emitted program contains no runtime operation-ID switch, target tag, per-call dispatch table, instance handle table, or handle lookup that selects among implementations.
A synchronous transfer lowers to its required source and target checks [STOR-6], at most one direct host call, one count or outcome check, and a cold outcome mapper reached only on failure.
That path performs no heap allocation, no copy of the transferred data, no global system lock acquisition, and no per-call signal-disposition operation.
The compiler wrapper is inlined, or any remaining call is shown to be immaterial, as a condition of qualification.
One-time per-invocation normalization belongs to the command bootstrap before entry rather than to any transfer: on the first native command targets that bootstrap owns the process and installs the ignored disposition for the write-to-closed-pipe signal, so a closed output destination reaches source as a recoverable outcome [ERR-4].
A program kind whose process the bootstrap does not own obtains an equivalent host guarantee under its own qualification and never changes a surrounding host process's signal policy.
This rule fixes the required emitted shape; the evidence establishing it is inspection of emitted code and symbols, not a machine-checked language judgment.

[TRAP-1] A trap in a program holding system resources retains [SCOPE-4] and [EFF-4] exactly: the runtime attempts the mandatory [DIAG-3] record and then aborts the whole process without unwinding and without running language cleanup; no status is produced [PROG-3].
No release, close, flush, detach, or completion action fixed by a system resource contract [STOR-3] runs after a contract violation, and no source-visible cleanup, handler, or recovery point exists.
Process-local memory, native descriptors, and every other process-local system object held at that moment are reclaimed by operating-system process teardown, which is a property of the host inside the [SCOPE-3] trusted computing base rather than a language cleanup guarantee.
External effects already performed are not rolled back: bytes already written remain written, an object already created remains created, and a persistent object or already-started external work retains the semantics its own family gives it.
A host that requires a Whitefoot instance to fail without ending its process runs that instance in a separate process.
Because a trap ends the owning process, no instance resource table, per-instance reaper, or pending-operation transfer is required, and none appears on a synchronous transfer path [QUAL-3].
Host-surviving in-process trap containment is a DEFERRED language amendment with its own delta [META-5].

[SYS-4] Each system type has exactly one kind, one `Sendable` judgment, and one `Shareable` judgment [CAP-1].
Every first-slice system type permits shared borrows, so permitting a shared borrow is not what separates the kinds.
An immutable value has no cursor, sequence position, or caller-visible mutation; owning storage does not make it a resource.
A shared capability owns no caller-visible cursor or sequence position that a later call consumes, and its shared operations may observe outside state or create an independent owned resource.
A stateful resource identifies one live stateful object; an operation that advances a cursor, fixes observable order, or otherwise changes its state takes `&uniq` or consumes the owner.

```wf-sys
| type | kind | Sendable | Shareable |
|---|---|---|---|
| `Args` | immutable value | yes | yes |
| `HostString` | immutable value | yes, on the command-lifetime argv backing | yes, on the command-lifetime argv backing |
| `RelativePath` | immutable value | yes, on the command-lifetime argv backing | yes, on the command-lifetime argv backing |
| `DirectoryRead` | shared capability | yes | yes |
| `ReadFile` | stateful resource | yes | no |
| `Output` | stateful resource | yes | no |
| `ExitStatus` | immutable value | yes | yes |
```

`ExitStatus` is Sendable and Shareable because it is an immutable command code with no interior state.
`HostString` and `RelativePath` are Sendable and Shareable because their backing is immutable and outlives the invocation [HOST-3, QUAL-2]; the judgment is a judgment about that backing, so a later string type with separately owned backing rederives both predicates from its own representation and inherits neither [SYS-9].
`ReadFile` and `Output` are not Shareable because a file cursor and an output publication order each have exactly one mutable owner; a later contract may add explicit lanes or consume `Output` into a publisher, and neither retroactively makes an original type shared.

These are declared capability predicates.
This specification defines no thread construct, so no program's acceptance depends on them; they fix what a concurrency layer may assume and what a later type may not inherit.

Kind is a type-contract distinction and introduces no writer-visible keyword.
`own`, `&'r`, and `&uniq 'r` [OWN-2] express every use of every kind.
A family operation that duplicates, splits, or attenuates a resource exists only when its alias, ordering, cleanup, and concurrent-use rules are complete; the first slice declares none, so no system value is duplicated, split, attenuated, or converted, and no integer right mask is exposed to source.

[SYS-5] Every system resource family declares one completion policy.
This specification defines exactly one: release-complete.
Under it, compiler-derived release is the complete language obligation for the type, and a source program needs no terminal operation to discharge ownership.
`Args`, `HostString`, `RelativePath`, `DirectoryRead`, `ReadFile`, `Output`, and `ExitStatus` are all release-complete, so this specification defines no exact-use checking obligation.

Two further policy classes are named and reserved without machinery.
Explicitly-abandonable means the type exposes a consuming abandon operation whose contract permits loss of unfinished external work, so abandonment is a source action rather than an accidental affine discard.
Completion-required means every normal or recoverable exit must consume the owner through a terminal transition.
This specification declares no type under either class and defines no operation, checker obligation, or diagnostic for either; naming them fixes the vocabulary a later buffered output, atomic replacement, pending operation, or child process must use rather than silently inheriting release-complete [SYS-12].

The consuming release action of each system type is exactly:

```wf-sys
| type | release action | release effect |
|---|---|---|
| `Args` | logical consume | none |
| `HostString` | logical consume of an inline lease | none |
| `RelativePath` | logical consume of an inline lease | none |
| `DirectoryRead` | at most one native close attempt | `external, blocks` |
| `ReadFile` | at most one native close attempt | `external, blocks` |
| `Output` | logical source detach | none |
| `ExitStatus` | logical consume | none |
```

A logical consume performs no host call, no target call, no handle lookup, no byte copy, and no external effect.
A native close attempt discards only the close diagnostic and never retries an ambiguous close: a consuming close invalidates the source handle on success and on error, because the native descriptor may already be closed and reusable.
`Output`'s logical source detach neither closes nor flushes the host descriptor [SYS-12].
Release of an outcome value is release of its components: `ArgError`, `Utf8Error`, `CopyError`, `Utf8CopyError`, `PathError`, `IoError`, and `ReadOutcome` have no release action and take no row above, and a `ReadOutcome` or `Result` carrying a system value releases that value by this table.

A release action is compiler-derived and explicit in the checked program [STOR-3, DIAG-2].
`flush`, `sync`, directory sync, atomic commit, and final handle release are different semantic operations; this specification declares none of them, and release is never a substitute for one.
Whole-process abort performs no release: a trap runs no language cleanup and returns no status [PROG-3, EFF-4, SCOPE-4], and the operating system reclaims process-local memory and handles while external writes are not rolled back.

[SYS-6] Each system operation declares its own outcome type; there is no shared outcome union.
An operation with exactly two outcomes returns a [PRE-1] `Result<T, E>` instantiation and declares no new constructor spelling.
The one operation with more than two outcomes declares one enum whose variant spellings carry its operation prefix, so no two operations compete for a constructor name in the whole-unit constructor domain [TYPE-6].
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
| `read_once` | `own ReadOutcome` |
| `write_once` | `own Result<u64, IoError>` |
| `exit_status` | `own ExitStatus`; total, no failure outcome |
```

`InvalidIndex` states that the requested argument index is not present and returns no value.
`Utf8Invalid` states that the host string is not valid UTF-8.
`CopyTooSmall(required)` and `Utf8CopyTooSmall(required)` state the exact length the destination range must have for the same call to succeed.
`Utf8CopyInvalid` states that the host string is not valid UTF-8.
`PathInvalid` states that the consumed host string is not a valid relative path and returns no value.
`ReadBytes(count)`, `ReadEnd`, and `ReadFailed(error)` are [SYS-8]'s three read outcomes.
On a successful `arg_get` the `Ok` payload is the requested `HostString`; on a successful length, copy, or write the `Ok` payload is the exact `u64` byte, encoded, or accepted length.

These error types are distinct nominal types and do not convert into one another [TYPE-4].
`propagate` [ERR-3] therefore chains only across operations that already share one error type: that is exactly `open_read` and `write_once` inside a function whose written result is `own Result<U, IoError>`.
`PathError`'s `PathInvalid` and `IoError`'s `InvalidPath` are deliberately different failures and never substitute for each other.

[SYS-7] `IoError` is the closed portable class set declared by [SYS-2].
Its thirty classes are the complete portable failure vocabulary of every system operation that can fail against a host, and the class is the sole portable semantic discriminator: exhaustive portable control flow branches on the class [ERR-2].
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

[SYS-8] `read_once`, `write_once`, `host_copy_bytes`, and `host_copy_utf8` are one-attempt operations over a caller-owned initialized `buffer<u8>` and a caller-written range.
Each takes only call-scoped borrows, so every resource and buffer owner remains with the caller on every outcome.

Range validation precedes every other action.
For `read_once` the range is `offset` and `capacity`; for `write_once` it is `offset` and `count`; for the two copy operations it is `offset` and `capacity`.
Overflow of the mathematical sum of the two range values in u64, an offset beyond the buffer's runtime length, or a range extending past that length traps as the operation-internal contract check retained by [OP-4] [ERR-4], before any host transfer, before any read of the source value or resource, and before any write of the destination.
A trap therefore leaves the resource, the source, and the buffer unchanged, and the target is never asked to validate a source pointer or a source range.

For a zero-length range, `read_once` and `write_once` report a count of zero and issue no host transfer.
A zero-length read is never reported as `ReadEnd`.

For a nonempty range, `read_once` and `write_once` make at most one host transfer attempt.
If that attempt reports progress, the operation returns that progress immediately and never hides a later failure by attempting again; a reported interruption is returned as `Interrupted`.
`read_once` returns `ReadBytes(count)` only for a count greater than zero, and `write_once` never returns `Ok(0)`: a host zero-length write is `Err(WriteZero())`.
A short success is not end of input; only `ReadEnd` states that no byte was available at the observed end.
Repetition, accumulation, and retry policy are ordinary source loops over these operations; this specification defines no read-exact, write-all, positioned, or vectored operation.

Buffer and cursor disposition is exact.
On `ReadBytes(count)` exactly the first `count` bytes of the requested range may have changed, every other byte of the buffer is unchanged, and the file cursor advances by exactly `count`.
On `ReadEnd` and on `ReadFailed` no byte of the buffer changes, because an attempt that made progress reports `ReadBytes` instead.
On every recoverable failure of `write_once` and of both copy operations the whole buffer is unchanged.
Every successful count is bounded by the caller's validated range, and the checked program retains that bound as a fact about the returned value [DIAG-2].
On `ReadBytes(count)` the count is at most the requested `capacity`; on a successful `write_once` the accepted length is at most the requested `count`; on a successful `host_copy_bytes` or `host_copy_utf8` the copied length is at most the requested `capacity`.
These are postconditions of the operations, not defensive obligations on source: a target returning a larger count violates its compiler-owned contract [QUAL-1], and source code neither checks nor branches on that possibility.

The two copy operations differ only after range validation succeeds.
`host_copy_bytes` performs the lossless transfer defined by [SYS-9] and has no failure mode beyond `CopyTooSmall(required)`.
`host_copy_utf8` first validates and measures the encoding and returns `Utf8CopyInvalid()` or `Utf8CopyTooSmall(required)` without writing any byte, and only then copies the complete encoding.
A successful copy changes exactly the requested destination prefix and leaves the rest of the buffer unchanged.

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
That count is exactly the `required` length a `host_copy_bytes` on the same host string reports, so a `host_copy_bytes` whose `capacity` is at least that count returns `Ok` with exactly that count, and the checked program retains that relation [DIAG-2].
For a target family whose native code unit is wider than one byte, what these two operations count and transfer is fixed by that family's target qualification; this specification defines it for no such family.
A qualification that narrows the result to what one string domain can carry, or that transcodes it silently, does not satisfy the lossless contract these two operations state.
The text route is defined on every qualified family.
On `Ok(length)`, a `host_copy_utf8` on the same host string neither returns `Utf8CopyInvalid()` nor, for a `capacity` of at least `length`, returns `Utf8CopyTooSmall(required)`, and the checked program retains that relation [DIAG-2].

`relative_path`'s construction, consumption, and retyping semantics are [PATH-1]; `PathInvalid()` returns no value and returns no `HostString`, and neither outcome allocates or copies a byte.

The one-host-string-type rule, the command-lifetime backing, the distinct owned-backing type for any other producer, and the no-implicit-retype consequence are [HOST-3]; release is a logical consume with no target call [SYS-5].
No system value stores an ordinary source borrow or needs a runtime handle-table lookup.

[SYS-10] `DirectoryRead` is a shared capability with one state.
It is live from its entry binding until its release and has no other transition: this specification declares no attenuation, duplicate, split, or explicit close operation for it, so no other state is reachable.

Opening creates aliases only downward.
`open_read` creates an independent `ReadFile` with its own cursor domain and does not alias the capability.
Two `DirectoryRead` values may denote the same directory object, and nothing infers separateness from a native handle or from a separate open.

Its completion policy is release-complete [SYS-5], on the same ground as `ReadFile` [SYS-11]: losing a close diagnostic on a read-only directory capability cannot invalidate an already opened file and cannot promise durability.

Any number of `open_read` calls may progress concurrently through shared borrows of one `DirectoryRead`, exposing no ordering relative to one another.
Each either creates its own `ReadFile` or fails, and none observes another's effect.

Resolution, process-equivalence, the no-emulation qualification rule, and the deferred confined root are [PATH-2]; the `command.cwd` instance is shareable for open operations.

[SYS-11] `ReadFile` is a stateful resource with one state.
`open_read` creates it live, with one cursor domain and one conservative filesystem-object alias domain.
A separate open does not prove a separate object, and this specification declares no duplicate, split, or positioned-lane operation, so multiple lanes over one file object are not reachable and sharing the handle is never inferred.
`read_once` is call-scoped and leaves both owners live on every outcome; its transfer, cursor, and buffer semantics are [SYS-8].

`ReadFile` is release-complete [SYS-5].
Compiler-derived release consumes the resource and may discard only a close diagnostic, which carries no guarantee about bytes already observed and no durability guarantee.
This specification declares no separate explicit-close operation.
A later consuming close may expose that diagnostic, but it must consume the owner on every outcome and may not change derived-release semantics.
Whole-process abort relies on operating-system teardown [SYS-5].

[SYS-12] `Output` is a stateful resource with one state.
The standard output and standard error entry bindings supply separate affine `Output` owners; neither is a shared global sink and neither carries a lock.
`write_once` performs at most one host output attempt [SYS-8], and its accepted count means exactly that the host operation accepted that prefix: it promises neither line atomicity nor storage durability.
Sequential calls across either owner preserve source order by the ordering rule that governs every external call, not by any aliasing analysis.
The checked program additionally retains the conservative fact that redirection may make the two owners the same sink [DIAG-2]; this specification defines no consumer of that fact, and it is retained so a later verified cross-resource reordering fact fails closed on this pair rather than treating two separate `Output` owners as disjoint sinks.

The target implementation adds no hidden userspace buffering, so every failure the host write itself reports reaches `write_once`.
`Output` is therefore release-complete [SYS-5]: compiler-derived release only detaches the source capability and reports nothing.
It does not close the host descriptor, it does not flush, and it makes no target call; operating-system process teardown closes the native descriptors afterwards.

That policy has one stated limitation.
A failure a host surfaces only at descriptor close or at writeback — delayed allocation, a network filesystem, a late out-of-space condition — is outside this specification's error model and can be lost, so a redirected command may return a successful `ExitStatus` after a failed writeback.
This is a stated limitation of the family contract, not a silently weakened guarantee.
Strengthening it is a later buffered or durable output type, which is completion-required [SYS-5] and must expose its own flush or finish operation; it does not inherit this policy.

A broken pipe reaches `write_once` as `BrokenPipe` through the bootstrap signal normalization [QUAL-3] fixes; a deployment the bootstrap does not own obtains an equivalent guarantee under its own qualification [QUAL-3].

Terminal control, color, and console mode are separate capabilities that this specification does not declare.
The mandatory trap record uses its own runtime channel [SCOPE-4, DIAG-3]; it never flushes an `Output` and source code cannot reach it.

[SYS-13] `ExitStatus` is an opaque immutable value carrying one portable command code.
`exit_status(code)` is its one constructor: it is total and pure, every `u8` is a valid command code, so the closed code range is 0 through 255 and there is no failure outcome, no allocation, no host call, and no external effect.
`ExitStatus` is release-complete and its release is a logical consume [SYS-5].

The type is opaque rather than an alias for `u8`.
There are no implicit conversions [TYPE-4] and every value's type is exactly what its producer fixes [TYPE-5], so without a stated constructor the command entry's returned value would be unwritable; keeping the type distinct also keeps an arbitrary integer from being returned as a command status, and matches how every other system type is fixed [SYS-2].

The target maps the returned code exactly onto the host process status.
Startup failure before entry and a trap are outside this mapping [PROG-3]: a trap performs no language cleanup and returns no status [EFF-4, SCOPE-4].

## 18. Obligation discharge: claims, entailment, and provenance (normative)

[CLM-1] `claim name: e because "text";` is a named runtime check.
`e` must have exact value mode and type `own Bool` under exactly the [OP-5] condition judgment, including the TYPE-7 implicit-read exclusivity: when `e` uses a borrow-mode or box/arena binding where its referent `Bool` value would be required, that use is rejected citing TYPE-7 and CLM-1 forms no candidate.
Every other exact-mode or exact-type failure is a hard error citing CLM-1 at the selected `expr` node, with `SourceCoordinate` equal to that node's complete checked half-open source extent.
A conforming claim is a runtime check in all build modes and is never elided; its checked-program disposition is always `retained` [DIAG-2].
If `e` is `False()` it emits the required trap record naming this claim [DIAG-3] and aborts [SCOPE-4, EFF-4]; if `e` is `True()` execution continues, and the passed fact enters the dominated continuation's fact state exactly as [ENT-3] admits it.
A `claim_stmt` syntactically exhibits `traps` [EFF-2] and does not count as delivery or must-divergence [GIVE-1].

The claim name is one IDENT and is not a declaration: it enters no [TYPE-6] domain, no [OP-1] reservation inventory, and no lexical lookup, and no source construct references it; its [DIAG-1] carrier classification is the claim-name carrier.
Because the name is outside the reservation inventory, a claim may be named `len` or `wrap`, while `trap`, `claim`, and every other exact fixed lowercase grammar atom remain unwritable as IDENT [FORM-3] — a chosen asymmetry (owner ruling 2026-08-07), not an accident.
Within one `fn_decl` every claim name is unique; a repeated spelling is a hard error citing CLM-1 at the later `claim_stmt` node.
The `because` STRING is the claim's justification: mandatory compile-time review data retained by the checked program [DIAG-2], absent from runtime behavior, and never semantics-selecting.
A claim is legal in exactly the statement positions [GRAM-4] admits; [FN-8]'s structural pass admits only ordinary lets and one final check, so a claim cannot appear in a `requires` block.
No predicate is illegal merely by operand provenance: a claim's own legality is judged by [CLM-2], while [PRV-2] and [PRV-3] constrain the downstream call argument or protected leaf rather than the claim and refuse only assertion-backed authorization of an external constrained subject.
A claim supporting no protected obligation is ungated, and a claim whose external operand occurs only as a bound, base, or unrelated goal operand remains legal.

[CLM-2] Claim lifecycle judgments are fixed by the entailment fragment under [ENT-1]'s monotonicity law, whose one enumerated non-monotone edge is this rule's refutation.
Redundancy and refutation are judged only for a predicate with comparison origin [ENT-3]; a conforming claim whose predicate has none — a constructed `True()`, a `band` result — is neither redundant nor refutable, is accepted, and traps whenever it evaluates false at runtime, exactly as today's `check` on the same expression, even though the passed claim establishes the predicate's signed decomposition members [ENT-3].
When the closed fact state at a `claim_stmt` [ENT-3] derives its predicate [ENT-4], the claim is redundant: the program remains accepted, the check still executes [CLM-1], and a conforming implementation reports one non-rejecting redundancy advisory naming the claim — an advisory is not a [DIAG-1] rejection, and a later specification version that proves more predicates therefore rejects no previously accepted program on that ground.
When the fact state is non-contradictory [ENT-4] and derives the predicate's exact negation, the program is rejected with a hard error citing CLM-2 at the `claim_stmt` node, carrying the claim name, the predicate, and the derived negation: a refuted claim is a defect found at compile time.
A claim whose trap record any execution produces is thereby demonstrated not to be a necessary truth; surfacing fired claims for reclassification is a toolchain contract in the [ERR-2] edit-list sense, not a language judgment.
Advisory channel and encoding are implementation-owned in this version; the advisory itself is required to exist.

[CLM-3] Any source `fn_decl`, generic or nongeneric, may carry the one optional fixed terminal `deny_claims` before its optional `program_kind`.
That terminal is ineligible for IDENT under [FORM-3].
Each marked concrete [FN-2] instance is one strict root.
The marker is compile-time policy only: it adds no effect, trap, runtime check, fact, type, mode, region, call convention, body, or lowering, and it neither removes nor changes any [CLM-1] claim or [OP-5] check.
A declaration without the marker is no strict root.

After every ordinary semantic and provenance judgment succeeds, form the finite concrete ordinary-user-call graph already used by [FN-9], retaining every checked call occurrence in source NodePath order, including calls and claims in structurally checked arms irrespective of value reachability or optimization.
Take the same SCCs and callee-before-caller condensation.
One direct claim identity is exactly `(concrete function instance, claim_stmt NodePath, claim name)`, independent of lifecycle disposition, reachability, or ledger use.
`DirectClaims(K)` is the union of all such identities in component K.
In callee-before-caller order, `MayClaims(K)` is `DirectClaims(K)` union the `MayClaims` set of every strictly outgoing callee component.
Sets are ordered by stable concrete-instance order, then NodePath, then name.
The closure of one strict root is exactly its root component plus every component reachable along outgoing edges; it includes the whole of each reached SCC and never follows an incoming edge into an unrelated caller.

A demanded component succeeds strictly exactly when its `MayClaims` set is empty, every protected obligation owned by the component discharges in its owning function's existing unasserted U state [OP-4, OP-2, ENT-6], every ordinary user-call requirement owned by the component discharges at that call in caller U [FN-8], and every strictly outgoing demanded callee component has a successful strict summary.
Calls inside one SCC consume no same-SCC summary; all members succeed or fail atomically.
These are finite queries over the already-produced view and DAG, not a body rewalk or another fixed point.
Component summaries are silent.
For one marked root, a nonempty `DirectClaims` set in its own SCC rejects at the least direct claim node; otherwise the first call in stable caller-instance then call-NodePath order within that SCC whose strictly outgoing callee component has nonempty `MayClaims` rejects there as an imported-claim event.
A direct claim in a reached unmarked component is therefore reported at the root-facing importing call, while a claim in the marked root SCC is reported at its own node.
When a downstream component instead fails a non-claim U judgment, only the actual OP-4 leaf or FN-8 call is reported; no caller-side summary event is created.
Multiple roots use the stable concrete-instance order fixed by [DIAG-1].

An ordinary caller outside the closure that calls a marked root remains ordinary but must also discharge that root requirement in its own U state [FN-8].
A marked program entry follows [PROG-3] and must discharge its requirement in the post-setup, pre-wrapper-check U state before the unchanged wrapper check and S4 body entry; this specification defines no foreign adapter.
Claim import is tested before a strict FN-8 judgment at the same call.
All strict roots and candidate S12 or delivery facts remain unpublished in one failure-atomic batch; any CLM-3, strict OP-4, or strict FN-8 event discards that batch and the prospective checked program.
Strict acceptance reads checked claim occurrences and call metadata directly from semantic scratch, never the checked-program `ClaimLedger` [DIAG-2], which is constructed only after successful finalization.

[ENT-1] The entailment fragment is a closed, deterministic, search-free derivation system fixed completely by this specification.
Its state is the L0 relation state plus [ENT-2]'s finite signed opaque goals.
The fixed judgments in this section are source-acceptance judgments: complete-state obligation discharge [ENT-6], claim redundancy and claim refutation [CLM-2], ordinary-call requirement discharge [FN-8], verified normal-return proof and view classification [FN-9], provenance classification [PRV-1], the call-argument gate [PRV-2], and the local constrained-subject gate [PRV-3] are post-resolution semantic judgments under [DIAG-1], identical in facts-on and facts-off compilation, and are not an optional optimizer-fact family. [SCOPE-2] is unchanged: every fact source [ENT-3] is an executed control condition, an executed runtime check, a requirement proved at an ordinary call or checked at a dynamic boundary before S4 admits it to a body, a declared allocation or type property, a constant, S11's compiler-owned structural consequence, or S12's machine-verified normal-result publication.
No source postcondition is trusted: FN-9 proves every selected exit, requires a nonempty selected-exit set, withholds same-SCC summaries, and subjects every candidate caller fact to the ordinary FN-8 and PRV gates before atomic publication.
The fragment is the deterministic checker derivation of [OP-4], [FN-8], [FN-9], and [DIAG-2] for the judgments this version attaches; a solver result never participates, and no implementation may strengthen, weaken, time-bound, or randomize the derivable set.
Two conforming implementations derive the same complete, unasserted, and S4-blinded fact states at every applicable point; the same FN-9 selected exits, aggregate dispositions, concrete-SCC order, and S12 establishment set; the same [PRV-1] class and symbolic dependency for every component; the same [PRV-2] result, write, demand, target, and event sets; and the same disposition for every obligation, claim, call goal, postcondition relation, local leaf, and call argument.
In a generic function, these judgments are made per concrete [FN-2] instantiation; a type or const generic in a requirement or postcondition is substituted first, and a const-generic constant term is judged at its concrete value, never symbolically.
The fragment joins the trusted computing base exactly as the type and ownership checkers do [SCOPE-3]; a wrong derivation is a compiler defect class, owned by testing, not a language hedge.
Version monotonicity of fact-source and closure strengthening is law with one enumerated exception: a later specification version may add fact sources and closure rules, and that strengthening removes none, so it never converts a discharged obligation, call goal, or selected-return relation into an undischarged one and never converts a claim into a redundancy-ground rejection.
The one exception is claim refutation: a strengthened fragment may newly derive a claim predicate's exact negation and reject under [CLM-2].
Activating [PRV-2] or [PRV-3] for an already attached protected family, attaching a new protected family, changing a [SYS-2] component from internal to external, or adding a callable publication surface is an amendment-level accepted-set change, not implementation strengthening.
Beyond those classes, this specification adds only FN-9/S12, the two stated unsigned S7 relations, [ENT-6]'s constant-operand overflow obligation family, and [ENT-5]'s value-if-only delivery, and retains the provenance gate.
No implementation may activate, expand, or reclassify any such judgment independently, and apart from an explicit specification amendment of those kinds no other entailment-fragment judgment may tighten acceptance across versions.

The [CLM-3] strict partition is one additional fixed source-acceptance judgment over the same finite semantic result, not a fact source or optimizer family.
Conforming implementations compute the same concrete ordinary-call occurrences, SCC condensation and callee-before-caller order, declaration markers, strict-root outgoing closures, direct claim identities, `DirectClaims` and `MayClaims` sets, demanded components, existing-U protected-obligation and call-goal dispositions, marked program-start disposition, and strict success summaries.
The judgment is per concrete [FN-2] instance, uses the already-produced U view and the same function-local derivation DAG, and introduces no solver, flow rewalk, second closure, negative fixed point, or new relation.
Facts-on and facts-off acceptance remains identical.
This partition is an explicit opt-in accepted-set tightening only for marker-bearing roots, plus the FORM-3 reservation of `deny_claims`; it changes no unmarked judgment and is the sole additional tightening authorized beyond the amendment classes enumerated above.

[ENT-2] The fragment constructs one flow state for one concrete function body and proof view at a time.
No caller fact is copied into a callee: an ordinary call judges its instantiated [FN-8] goal in the caller's entering state, the callee body begins with its own proved requirement as [ENT-3] source S4, and only a separately FN-9-verified earlier-SCC summary may establish its instantiated normal-result relation back in the caller.
A fragment type is one member of the closed integer set [OP-2]; relations are over mathematical values, so relations between terms of different fragment types are well-formed and are created only by the sources and flow transports [ENT-3, ENT-5] admit.

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root `pbase` IDENT resolves to any `let_stmt` binding, a `for_stmt` binder, a `param`, a requires-clause local, any match binder regardless of its [OWN-13]-derived mode, or a named const [CONST-2], formed with any number of field-selection `psuffix`es and `deref` wrappings and no subscript suffix, whose final selected type is one fragment type; (b) a length term `len(P)`, of fragment type u64, where P is a place formed under the same restriction whose final selected type is `array<T, N>`, `slice<'r, T>`, or `buffer<T>`; (c) a constant — the mathematical value of an integer literal or of an integer-typed named const, or symbolically an in-scope integer-typed const-generic parameter; (d) one of the two compiler-owned u64 capture terms belonging to an admitted `for_stmt`, identified exactly by `(that for_stmt's NodePath, lower)` or `(that for_stmt's NodePath, upper)`; (e) the one compiler-owned symbolic result datum of an admitted FN-9 block while its RelationTemplate is formed, identified by that selector declaration and fragment type; or (f) the distinguished zero term Z, used only to carry constant bounds and S7's exact mathematical-zero disequality.
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
It has the finite structural identity fixed there, is neither a place nor an L0 term, has no direct or expanded source goal origin, and therefore cannot be established by naming the original subscript again.
Goal equality is exact tree equality and therefore may hold across two requirement occurrences or concrete callee instances only when their substituted typed trees are identical.
The finite goal universe of one concrete function is exactly the goals formed from its written Bool conditions, checks, claims, requirement S4, and ordinary-call requirements after the finite expansions [ENT-3] admits.

A signed opaque fact is exactly `+G` or `-G` for one concrete goal G, meaning that exact whole expression evaluated respectively true or false.
It carries no child facts and receives no Boolean-algebra closure.
If G's complete root is exactly one comparison origin relation R under [ENT-3], `+G` has the exact L0 projection R and `-G` has R's exact negation; a non-comparison root has no L0 projection.
The signed fact and its projection are distinct members of one combined state and have the supports [ENT-5] fixes.

An atomic fact is one difference bound `t1 - t2 <= c` (t1, t2 terms, c a mathematical integer) or one disequality `t1 != t2`.
Source relations normalize exactly: `a <= b` is `a - b <= 0`; `a < b` is `a - b <= -1`; `a = b` is the bound pair `a - b <= 0` and `b - a <= 0`; `a >= b` and `a > b` swap operands; `a != b` is one disequality.
A constant operand folds through Z: `a <= 7` is `a - Z <= 7`.
Implicit facts hold at every program point: every term t carries the reflexive bound `t - t <= 0`; every term t of fragment type T carries `t - Z <= max(T)` and `Z - t <= -min(T)`; every length term over a place of type `array<T, N>` carries the equality `len(P) = N` (both bounds), with concrete N a constant and const-generic N a symbolic constant term.

[ENT-3] The fact state is defined constructively over the conservative structural normal-control graph [FN-1]: each source below establishes its L0 and signed-goal facts at its stated point; facts flow forward along normal edges; kill events apply on the edges where [ENT-5] places them, with scope-exit kills applied before any join; merge points take the [ENT-5] join and loop heads the [ENT-5] loop rule; and the state queried at any point is the [ENT-4] closure of that flow.
retired: S8
Dominated straight-line establishment is a consequence of this construction, not a second definition.
Nothing else is a fact: an `ensures_block` is only an FN-9 proof obligation, never a trusted source; no struct invariant, writer-stated or inferred loop induction, inferred summary, or unverified user-function result exists.
S11 is only the compiler-owned consequence of the counted operations [FN-1] actually executes, and S12 exists only from a separately verified earlier-SCC summary under the publication formula below.
Provenance [PRV-1] is a separate judgment over finite value and storage components, not a fact: it establishes and kills no relation or signed goal, and no [ENT-4] answer depends on it.

A comparison origin is defined first.
An expression has comparison origin R when (a) it is a call to one of `ieq`, `ine`, `ilt`, `ile`, `igt`, `ige` [OP-2] whose two operands are each a term or constant, R the corresponding relation over them; or (b) it is a bare IDENT naming a `let` binding of type `own Bool` whose initializer right-hand side satisfies (a) with relation R, no [ENT-5] kill event (a)–(d) applies to a fact supported by an operand term of R on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`, user-function results, and deeper indirection chains contribute no L0 comparison origin in this version; an established Boolean goal contributes relations only through the members of its signed decomposition set.

A Bool expression has a direct goal origin G when its completely typed expression consists only of non-consuming place datums, typed literals, named const datums, and calls to or infix spellings of pure, total, non-trapping operation-table rows, with exact tree identity as [FN-8] fixes.
Construction, a user-function or system call, a subscript, a move or borrow, a trapping or partial operation, and any other expression shape has no goal origin.
Starting from a direct goal, its complete origin expansion recursively replaces an ordinary-let datum by that binding's unique defining right-hand side exactly when the right-hand side itself has such a typed pure/total origin, the binding is no `set` target on any path from that initializer to this use, and no [ENT-5] kill event applies to the replacement's support on any such path.
Expansion continues to a fixed point and is all-or-nothing for every eligible leaf; it never performs an algebraic rewrite.
The goal-origin set is the direct goal plus that one complete valid expansion when it differs.
Thus a condition binding's own Bool value and its still-valid computation origin are both retained: a later write to an origin place kills the expanded goal but not the already-computed binding goal, while a write to the binding kills the latter normally.
Clause-local expansion in FN-8 is unconditional because the admitted block contains no mutation.

Signed Boolean decomposition applies at every establishment of a signed goal fact by the sources below.
The decomposition set of `+G` whose complete root is `band(A, B)` is `+A` and `+B` together with each member's own decomposition set; the decomposition set of `-G` whose complete root is `bor(A, B)` is `-A` and `-B` together with each member's own decomposition set; the decomposition set of `+G` or `-G` whose complete root is `bnot(A)` is respectively `-A` or `+A` together with that member's own decomposition set; a `bxor`, `eeq`, `ene`, comparison, datum, or non-Boolean root has the empty decomposition set — in particular `-band` and `+bor` carry only genuinely disjunctive content and establish nothing about a child.
When a source establishes `+G` or `-G`, it establishes every member of that signed decomposition set at the same point; each member is one concrete goal under [FN-8]'s structural identity, and each member whose complete root is one comparison call admitted by comparison-origin shape (a), whose operands are each an admitted term, constant, or `len(P)` length term, independently establishes that exact relation under `+` and the relation's exact L0 negation under `-`.
A member's support is the ordinary [ENT-5] signed-goal support of its own complete typed expression; kill events, scope exits, joins, and the loop rule apply to each member independently of its parent.
Decomposition is a finite structural walk of the established goal's tree: it performs no algebraic rewrite and no children ever establish or derive a parent.

The sources are:

[ENT-3.S1]
- S1 (branch facts).
At an `if_stmt` or `value_if`, each goal G in the condition's goal-origin set is established as `+G` at the then-block's entry and `-G` at the else-block's entry; for an else-free `if_stmt`, `-G` is established on the false edge, which joins the then exit at the continuation [ENT-5].
Independently, when the condition has comparison origin R, R is established at the then entry and R's exact negation at the else entry or false edge.
L0 negation is exact over mathematical integers: the negation of `a - b <= c` is `b - a <= -c - 1`; the negation of `a = b` is `a != b` and conversely.
[ENT-3.S2]
- S2 (check facts).
After `check e else trap "…";` [OP-5], each goal in `e`'s goal-origin set is established with positive sign on the normal continuation; when `e` also has comparison origin R, R is established there independently.
[ENT-3.S3]
- S3 (claim facts).
After `claim n: e because "…";` [CLM-1], establishment is exactly [ENT-3.S2]'s.
[ENT-3.S4]
- S4 (requires facts).
At a concrete function-body entry, its complete instantiated [FN-8] goal G is established as `+G`.
When and only when G's complete root is one comparison call admitted by comparison-origin shape (a), whose operands after template and call substitution are each an admitted term, constant, or `len(P)` length term, that exact relation R is also established.
Beyond that projection, only the members of G's signed decomposition set and their projections are established; no other child of any goal is established.
S4 is the admitted-body axiom justified by every ordinary caller's static discharge or the successful dynamic boundary check [PROG-3, GATE-1]; no callee-entry prologue executes.
[ENT-3.S5]
- S5 (copy and conversion equalities).
An `ordinary_let_rhs` establishes at its binding: for `let x = lit;`, x = value(lit); for `let x = p;` with p a term of type T, x = p; for `let y = cvt<Src, Dst>(p);` with (Src, Dst) a total pair [OP-6] and p a term or constant, y = p — `cvt` keeps its written type pair [TYPE-5].
[ENT-3.S6]
- S6 (length facts).
`let b = buffer_new(n, v);` and `let b = buffer_vacant<T>(n);` each establish len(b) = n on the normal continuation [OP-9], n read as term or constant.
`let m = len(P);` for a tracked P establishes m = len(P).
`let s = slice_of…(&'r P);` for a tracked P establishes len(s) = len(P).
[ENT-3.S7]
- S7 (constant-offset arithmetic).
For `let s = p +wrap k;` with p a term of type T and k a constant in either operand position, when the closed state at that point derives `min(T) <= p + k` and `p + k <= max(T)` (as bounds on p through Z), s = p + k is established; `p -wrap k` with constant k establishes s = p - k under the dual range condition.
For `p + k` and `p - k` with constant k, s = p ± k is established on the normal continuation unconditionally: the site is a constant-operand-class call whose discharged overflow obligation is the proof [OP-2, ENT-6].
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
- S10 (boundary count facts).
For a `match_stmt` or `value_match` whose scrutinee is directly a call to `read_once`, `write_once`, `host_copy_bytes`, or `host_copy_utf8` [SYS-2, SYS-8], or a bare IDENT naming a `let` binding of the call's outcome type initialized by such a call under the same no-kill, no-`set` path discipline as S7's checked-arithmetic origin: with k the actual bound to the call's bounding parameter — `capacity` for `read_once`, `host_copy_bytes`, and `host_copy_utf8`; `count` for `write_once` — read as a term or constant, where no [ENT-5] kill event applies to a fact supported by k on the path to the match, the `ReadBytes(count: w)` arm of a `read_once` match and the `Ok(value: w)` arm of the other three establish w <= k at arm entry; every other arm establishes nothing.
These facts carry the same trust class as S6's allocation-length equality — a declared operation contract, never a writer statement.
The three [SYS-9] relations are retained checked-program facts and are not L0 fact sources in this version.
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

The opaque component retains exactly the established signed facts — Boolean decomposition happens at [ENT-3] establishment, never here — and receives no closure, composition, or implication rule.
`+G` is derivable when that exact positive fact is present or when G has an exact comparison projection R and L0 derives R; `-G` is derivable when that exact negative fact is present or when G has that projection and L0 derives R's exact negation.
Deriving the two children of a Boolean operation never derives its parent, and derivability never decomposes: only an established parent establishes its members, at its establishment point.

The combined state is contradictory when L0 derives `t - t <= -1` for any t or when both signs of one exact goal are derivable.
At a contradictory point every L0 relation and both signs of every goal in the finite universe are derivable, every obligation, call goal, and FN-9 selected-return relation is discharged, and no call goal, selected-return relation, or claim is refuted.
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

An S12 relation, a narrow-receiver relation, and a relation transported through `value_if` have exactly the ordinary L0 support of their terms after the route's stated substitutions.
The callee summary reference, proof view, call or delivery edge, pre-transfer substitution record, and a result or payload binder already replaced by its receiver are checked metadata, not additional support.
A route whose substitution leaves a non-[ENT-2] operand never creates an L0 fact.

Independently of relation flow, FN-9 entry-image stability begins live for each referenced parameter datum at function-body entry.
The same overlap, holder, consume, effect, scope-exit, and counted-continuing-kill classifications below permanently invalidate it; for a `len(P)` datum the element-storage exception is the same as for ordinary length support.
A structural merge retains stability only when every reaching input retains it, and a loop head removes stability for every datum a continuing kill may invalidate.
Neither contradiction, re-establishment of a fact, assignment of an equal value, nor a later iteration restores stability.
This metadata creates no snapshot, term, relation, signed goal, or runtime action.

A fact dies at the earliest of: (a) a [SET-1] `set` or [SET-2] `replace` commit whose resolved target [SET-1, SET-2, OWN-5] overlaps, under [OWN-7]'s overlap relation, the resolved place of any support member, or the compiler-owned update of a `for_stmt` binder when that binder is a support member — because a length term's support is its viewed place's non-element root path, a whole-place replace of a buffer or of any prefix of it kills that buffer's length facts, while an element-position replace, like an element write, kills none; (b) a call — user function, table operation, or system operation — one of whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller place or origin set containing a place that overlaps [OWN-7] the resolved place of any support member; the projection is exactly [EFF-2]'s, so a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place, and a call whose row carries no `writes` kills nothing; (c) a consuming use [OWN-1] of any support member's root; (d) an edge leaving the region of any borrow holder in its support, leaving the lexical scope of any support binding, or leaving the owning counted construct of any capture term in its support, region exit [OWN-3] included.
Scope exits are edge events: kills (c) and (d) apply on every edge leaving the scope, before any join at that edge's target is taken — mirroring [STOR-3]'s edge-carried releases — so no arm-local or block-local fact survives its scope into a join under any reading.

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
A non-bare, projected, consuming, computed, constructed, call, subscripted, literal, named-const, const-generic, capture, Z, requires-local, wrong-mode, or wrong-type delivery forms no image; the value still follows ordinary GIVE-1 semantics.
A `value_match` forms no delivery image under any source shape.

At the receiving `let` continuation, ordinary fact flow and its ordinary branch join remain unchanged.
Separately join one delivery image from every reaching `give` edge of the `value_if`, in edge NodePath order, after the substitutions and kills above.
When at least one image is non-contradictory, contradictory images are neutral and the non-contradictory images retain for each ordered term pair the weakest (largest-constant) bound held by all and each disequality held by all; a relation missing from one such image is not delivered.
Hence images containing `x < 8` and `x < 128` establish `x < 128`, not nothing and not `x < 8`.
An all-contradictory image set is contradictory; an absent eligible relation on a non-contradictory edge contributes an empty image and prevents delivery of that relation.
Add exactly the joined L0 relations to the receiver's ordinary continuation state and close once.
This transport reads no pre-existing fact on x, forms no inverse `x ↦ d`, copies no unrelated relation, and creates no runtime operation.

Joins: at the continuation of a `match_stmt` or `value_match`, the fact state is the join of the states on every arm exit edge reaching that continuation on the conservative structural graph [FN-1], each taken after that edge's scope-exit kills and then closed [ENT-4]; an arm every path of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s error edge contributes nothing there.
In any nonempty join with at least one non-contradictory input, a contradictory all-derivable input imposes no constraint.
Over the non-contradictory inputs, the L0 join keeps for each ordered term pair the weakest (largest-constant) bound held by all and each disequality held by all; the opaque join keeps one signed fact exactly when that identical goal and sign are held by all.
The join of closed states is closed.
A nonempty join whose every input is contradictory, and an empty join with no reaching edge, are each the contradictory all-derivable state.
At the continuation of an `if_stmt` or `value_if`, this same join is taken over every branch exit edge reaching that continuation — for an else-free `if_stmt`, the false edge is such an edge — each after its scope-exit kills and closure; a branch every path of which leaves by `return`, `break` to an enclosing loop, or `propagate`'s error edge contributes nothing there.
The continuation of a `loop_stmt` uses the same join over its `break` edges.
A `loop_stmt` with no `break` naming its label has an empty join and therefore the contradictory state, consistent with that continuation being unreachable in truth while the conservative graph keeps it reachable.
A `propagate` right-hand side's `Err` edge leaves the function; its normal continuation keeps the preceding state subject to the initializer call's own kill events (b) and (c), and its binder gains no fact.

The continuation of a `for_stmt` is the join of its structural false-header edge and every `break` edge naming that counted label, each taken after all binder, capture, and body-scope exit kills and then closed.
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
This version attaches exactly two obligation families.
The first family: for every source subscript `P[i]` — read, write, and [SET-1] target position alike — the bounds obligation `i < len(P)`, normalized `i - len(P) <= -1`, at that subscript's `psuffix` node, one obligation per subscript in a chain, where `i` is the offset atom whose exact type [OP-4] fixes as `own u64`, so both sides are u64-typed and the relation is over their mathematical values.
The second family: for every bare-operator `+`, `-`, or `*` call in [OP-2]'s constant-operand class, the overflow obligation that the call's exact mathematical result belongs to the selected type T's value set, at that call's `infix` node.
The overflow obligation normalizes to exactly two conjuncts — ordinal zero the upper bound and ordinal one the lower bound — each one difference bound between the non-constant operand read as a term and Z with one checker-computed constant, folded exactly as follows over mathematical integers, with floor and ceiling the exact-quotient roundings toward negative and positive infinity.
For `t + c` and `c + t` with constant c: `t - Z <= max(T) - c` and `Z - t <= c - min(T)`.
For `t - c`: `t - Z <= max(T) + c` and `Z - t <= -min(T) - c`.
For `c - t`: `t - Z <= c - min(T)` and `Z - t <= max(T) - c`.
For `t * c` with c > 0: `t - Z <= floor(max(T)/c)` and `Z - t <= -ceil(min(T)/c)`.
For `t * c` with c = 0: both conjuncts are `Z - Z <= 0`.
For `t * c` with c < 0: `t - Z <= floor(min(T)/c)` and `Z - t <= -ceil(max(T)/c)`.
For two constant operands with exact mathematical result z: both conjuncts are `Z - Z <= 0` when z belongs to T's value set and `Z - Z <= -1` otherwise.
The complete-state base judgment discharges a conjunct exactly when the closed complete fact state at that node derives it [ENT-4, ENT-5], and discharges the obligation exactly when both conjuncts discharge.
Failure of that base judgment is the [OP-2] rejection; its diagnostic renders the residual of the least undischarged conjunct as exactly: the non-constant operand's canonical source bytes, then ` <= `, then the conjunct constant in decimal, for ordinal zero; the negated conjunct constant in decimal, then ` <= `, then the operand's canonical source bytes, for ordinal one; and, for a ground conjunct, the exact decimal mathematical result, then ` outside `, then the selected type's spelling.
The overflow family attaches base discharge only: it creates no [PRV-2] or [PRV-3] protected demand, no provenance event, and no runtime operation in this version.
An operand that is not a term or constant leaves each non-ground conjunct underivable, and the one-rebinding fallback stated below for a subscripted offset atom applies identically to a subscripted class operand.
Failure of that base judgment is the [OP-4] rejection, forms no provenance demand or event, and publishes no checked program; its diagnostic renders the residual as exactly: the offset atom's canonical source bytes, then ` < len(`, then the base place's canonical source bytes, then `)`.
The mechanical fix for a base failure is one dominating claim or branch establishing the relation — in canonical ANF, one `let` binding `len(P)` followed by one `claim` on, or `if` over, the admitted comparison [CLM-1, ENT-3].
After base success, a [PRV-2] or [PRV-3] provenance rejection makes the assertion half unavailable: the writer uses a dominating value branch whose false edge takes the domain outcome, or restructures so the external value no longer occupies the constrained-subject position.
For an offset atom that is itself a subscripted place — legal under [GRAM-5]'s place grammar but no term under [ENT-2] — the base fix first rebinds that inner read through one ordinary `let` (and, where the element type is narrower than u64, one total `cvt` [OP-6], both S5-tracked), making the offset a term whose own inner obligation is discharged the same way.
With at most that one rebinding step per nested offset, the fallback always closes base discharge, at a per-site cost from zero where facts already prove the bound to one retained runtime check where none do; it does not by itself satisfy the provenance gate.

For checked metadata only, each concrete obligation has protected-leaf identity `(concrete function instance, exact obligation-occurrence NodePath, normalized conjunct ordinal)`.
The bounds relation has one conjunct at ordinal zero; the overflow relation has its upper conjunct at ordinal zero and its lower conjunct at ordinal one.
A requirement occurrence has identity `(the same form of concrete function instance, final-check NodePath, 0)` [DIAG-2].
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
A system result adds no parameter datum and seeds each plain or direct payload component's unconditional-external bit exactly as [SYS-2]'s closed table fixes; an internal component seeds no bit.
Storage, result, write, and user-call component propagation are solved first to a least fixed point over the finite concrete instances and then frozen.
Recursion and mutual recursion in this component stratum therefore use that fixed point, not traversal order or a stored witness path.
Only after the component pairs freeze does the direct-demand and requirement-bridge stratum below inspect them; that second stratum never feeds a bit or datum back into a component pair.
The ephemeral actual-value datum of FN-8 is separate from this dependency judgment: its originating checked actual still carries the ordinary root, offset, and operand dependencies defined here.

For a protected leaf, its constrained subject is only the offset value `i` in `i < len(P)`.
Its subject parameter datums are exactly the parameter-dependency set of that value at the obligation.
The base P, `len(P)`, a bound, a write target, and every other operand mentioned by a requirement goal contribute no subject datum merely by being mentioned.
A leaf with no subject parameter datum still has its protected-leaf and structural bridge identity; an implementation must not manufacture a datum from a bound, base, or another goal operand.

For the active gate, the **complete state** is the ordinary [ENT-3] flow and closure used by the base [OP-4] and [FN-8] judgments.
The **unasserted state** U is that flow recomputed with S2 and S3 establishment disabled and every other source, kill, join, loop rule, and closure unchanged.
The **S4-blinded state** B is U with both the function's positive S4 goal and its exact L0 projection, when any, omitted at body entry.
Only a leaf whose complete-state base judgment succeeds reaches the local demand generator.
If B discharges the leaf, add no demand.
Otherwise, if the subject component's unconditional-external bit is true, create the local [PRV-3] candidate regardless of U and regardless of whether the component also carries parameter datums; retain those datums only as explanations and add no direct demand or bridge tuple for that leaf.
Only a component whose unconditional bit is false reaches the remaining partition: if U does not discharge the leaf, add one direct `(subject parameter datum, leaf)` demand for each subject datum; if U discharges it while B does not, add the structural pair from that function's requirement occurrence to the leaf and add one `(requirement occurrence, subject parameter datum, leaf)` bridge tuple for each subject datum.
A false bit with no subject datum is internal and creates no rejection or caller-visible target.
A full-state failure remains [OP-4], forms none of these members, and publishes no checked program.
A function with no S4 requirement cannot distinguish U from B.

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
A kind-declaring `command` entry has no caller to continue the latter bridge, so a false-bit selected actual creates no local gate event there.
No synthetic external parameter datum is introduced.

After the component stratum converges and freezes, take direct-demand, bridge, call-target, and event composition together to a second least fixed point over the finite concrete function instances, frozen component pairs, requirement occurrences, full-state-accepted calls, parameter datums, and protected leaves.
Complete/U/B outcomes and every tested unconditional bit are fixed inputs to this stratum; its transfers only add set members, so a false-bit premise never later becomes true and requires no retraction.
Direct, recursive, and mutually recursive demand paths therefore converge independently of traversal order, while a recursive component with no local protected-leaf seed remains empty.
Multiple datum or leaf explanations for one actual argument remain distinct targets, but [PRV-2] coalesces them into one event per call and argument.
Witness paths and tie-breaking predecessors are selected only after this second convergence and cannot change either lattice.
The checked program retains the converged components, direct demands, structural pairs, bridge tuples, call links, complete/U/B outcomes, target sets, and deterministic finite witness predecessors [DIAG-2].
An unconditional-external bit is never replaced by or propagated as parameter-only demand metadata: it terminates at its local leaf under [PRV-3], or at its call argument under [PRV-2] for a direct demand or a B-failing bridge, retaining any parameter explanations only for diagnostics.
At a kind-declaring `command` entry, each labelled input is unconditionally external [PRV-1]; a B-failing direct local leaf whose subject carries that bit is owned by PRV-3, while a B-failing inherited bridge whose selected actual carries that bit is owned by PRV-2 at that call's argument.
This active bridge and gate add no runtime operation, fallback check, trusted assertion, or optimizer consequence.

For [CLM-3], the unasserted U state is exactly the unasserted state U above, retaining S4 after its independently proved incoming boundary.
Each demanded protected leaf queries its existing normalized relation in U and each demanded ordinary-call goal queries its existing instantiated goal in caller U; successful queries retain their already-produced U derivation roots.
A marked program-start requirement instead queries U before its wrapper check or S4 exists.
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
A [SYS-2] result or writable component begins with exactly its table-fixed unconditional bit and no parameter datum.
These entry and system components are the only unconditional external origins.

The complete transfer is [ENT-6]'s positive dependency transfer applied componentwise to that pair.
In particular, storage is flow-insensitive per binding and whole root; every initializer, overlapping `set`, and projected call write joins, and no later flow subtracts an edge.
A selected place read joins its root and every explicit subscript-offset atom in the resolved place, field selection preserves the accumulated pair, and `len(P)` is internal.
A `set` target's address contributes nothing to the stored value or root; only its right-hand side does.
Literals and named consts are internal.
Ordinary table-operation results join their value operands, checked arithmetic and `cvt` carry those operands only into `Ok(value:)` while a tag-only `Err(error:)` is internal, total `cvt` and `reinterpret` preserve, and construction, matching, `give`, propagation, return, and counted-binder initialization follow the exact component transfers already enumerated by ENT-6.
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
The same partition applies inside a kind-declaring `command` entry, which has no ordinary caller to continue a false-bit bridge.
Each record retains the callee parameter datum, demand kind, exact protected leaf, every bridge predecessor, and the nonpropagated companion parameter datums in the parameter order above.
If `Targets(c, q)` is nonempty, the compiler emits exactly one hard rejection event for that `(call, q)` pair, citing PRV-2 with `SourceNode` at the existing argument `atom` node and `SourceCoordinate` equal to that atom's complete checked half-open extent.
A second datum, leaf, or route at the same argument enlarges the retained target set but creates no second event.
Events at different argument ordinals remain distinct.
A call whose full goal is refuted or unproved is only [FN-8], creates no target, and never reaches this judgment.
A source call to the unlabelled `main` follows this ordinary rule; a kind-declaring entry remains uncallable [FN-7].

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
The protected leaf's concrete instance, obligation NodePath, and conjunct ordinal follow that route key.
A remaining tie between concrete instances uses one implementation-defined but stable deterministic instance order fixed for that compiler executable and independent of hash iteration, worklist order, allocation identity, or worker scheduling.
At the terminal boundary, the PRV-1 predecessor suffix follows only edges carrying the true unconditional bit to its labelled-entry or [SYS-2] origin, minimizes component edges, and then compares the complete sequence of carrier NodePaths fixed by PRV-1 and their selectors lexicographically; a companion parameter path is not eligible as that suffix.
Reconstruction records visited complete demand-state identities, so `direct(F, d, L)` and `bridge(F, R, d, L)` remain distinct even at the same function, datum, and leaf, while revisiting one identical full state cuts the cycle and recursion never appears as an infinite witness.

The event payload retains the complete ordered `Targets(c, q)` set, the selected leaf's [ENT-6] residual, and one rendered chain from that leaf backward through requirement occurrences, callee datums, and call boundaries to the rejecting actual, then through that actual's PRV-1 predecessors to its labelled-entry or [SYS-2] origin.
For a direct demand, its legal repair is a real branch in the protected leaf's owning body that establishes the residual and takes the domain outcome on the false edge.
For a requirement-bridge target, the branch instead establishes the complete bridged call goal in the rejecting caller's unasserted state before that call.
Either kind may also be repaired by restructuring the route so the external value no longer reaches the protected constrained subject.
A body `check`, `claim`, fallback callee prologue, or retained wrapper check is not a repair.

[PRV-3] This rule owns only a local protected leaf, including a leaf local to a program entry.
The [ENT-6] complete-state base judgment runs first.
If it fails, [OP-4] is the sole rejection and no PRV-3 candidate exists.
After base success, the constrained subject of the sole current obligation family is exactly the offset value `i` in `i < len(P)`.
If B discharges the leaf, no provenance demand remains.
If B does not discharge it and the subject's PRV-1 pair has a true unconditional-external bit, the leaf is a hard rejection citing PRV-3 with `SourceNode` at its existing subscript `psuffix` node and `SourceCoordinate` equal to that node's complete checked half-open extent, regardless of U and regardless of whether the pair also carries parameter datums.
Those datums remain diagnostic explanations but create no direct demand, bridge tuple, or second event.
Only a false-bit pair reaches the remaining partition.
An empty parameter set is internal and creates no rejection or caller-visible target.
With a nonempty parameter set, failure in U retains the direct demand for [PRV-2], while success in U followed by failure in B retains the S4 requirement bridge for PRV-2; neither case is a local PRV-3 rejection.

A kind-declaring `command` entry follows the same partition and has no ordinary caller that can prove its requirement.
Every labelled input has a true unconditional-external bit, so a direct entry-local leaf whose subject carries that bit and which B does not discharge is the same local PRV-3 rejection whether U fails or succeeds only because of that entry's S4 axiom; B success ends the demand.
An inherited leaf reached through an entry-body call remains a call-argument judgment and is owned only by [PRV-2].
The compiler-owned wrapper evaluation is still executed exactly once for every unrelated entry requirement [FN-8, PROG-3], but neither that runtime check nor the S4 axiom authorizes this leaf.
A real source branch in the body remains S1 in U and B and may discharge it.
This special entry disposition adds no foreign adapter, alternate error protocol, source surface, or second body.

The unasserted state removes exactly S2 body-check and S3 claim establishment.
S1 branches, S4 except where the entry bridge is explicitly blinded, S5, S6, S7, S9, S10, S11, every kill and join, and [ENT-4] closure remain unchanged.
Thus a `check` or `claim` may not authorize an external constrained subject, while an internal subject may continue to use either; a claim supporting no protected leaf is untouched.
Provenance of `P`, `len(P)`, a bound, a target address, or another goal operand does not gate the obligation, because none is its constrained subject.

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

fn sign_of(x: own i32) -> own Sign pure {
  doc "Conditional value produced by returning from branches (canonical for return position).";
  if ilt(x, 0_i32) {
    return Neg();
  } else if ieq(x, 0_i32) {
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
    check ieq(v, 42_i32) else trap "arithmetic drift";
  }
  return unit;
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
