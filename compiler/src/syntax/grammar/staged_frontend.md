Staged v0.18 frontend-contract snapshot. These are the exact canonical-format,
lexer, and grammar contract sections of the staged specification candidate
(governance/spec-evolution/kernel-spec-v0.18-candidate.md) that the staged
grammar tables in staged.rs describe. The whitefoot-grammar verifier compares a
proposal against this snapshot byte-for-byte and fails closed on any other
contract. Superseded and deleted when a v0.18 specification is activated.

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting. Non-canonical input is a hard error; the toolchain never auto-formats. Unknown constructs are hard errors (conservative extension).

[FORM-2] Each source file is UTF-8. Once every source has passed raw lexical formation and the complete compilation unit has one derivation, each source owns one ordered derivation forest: exactly the top-level `item` subtrees under the single compilation-unit `program` root whose terminals belong to that source, in source-local item order. A source forest is not a second `program` node, and a source with no items owns an empty forest. That source's canonical bytes are exactly the result of rendering its forest by the following rules. The input bytes must equal that rendering byte for byte; the toolchain does not normalize or rewrite input. A source that has no complete `item*` derivation is rejected by its owning lexical or grammar rule before this forest-format comparison, and no tree or forest is fabricated [DIAG-1].

Outside terminal interiors, lines end only with LF and formatting bytes are only ASCII space and LF. There is no CR, tab, trailing horizontal whitespace, leading blank line, or blank line inside a top-level item. A nonempty source has exactly one empty line between consecutive top-level `item` nodes and no trailing blank line; its final nonempty line ends with exactly one LF. A source containing zero items is exactly one LF. Terminal interiors retain their exact bytes and are checked by their owning FORM rule.

The left-attachment set contains `(`, `[`, `<`, `&`, and `.`. The right-attachment set contains `)`, `]`, `>`, `,`, `;`, `.`, `:`, `(`, and `<`. Between two consecutive terminals on the same line, emit zero bytes when the left terminal is in the left-attachment set or the right terminal is in the right-attachment set; otherwise emit exactly one ASCII space. Thus function headers are `fn f()`, `fn f<T>()`, and `fn f ['r]()`; generic and square-bracket interiors are compact; `](` and `>(` are attached; and commas and colons attach to their left operand and have one space before the grammar-required following element. Examples include `Result<i32, Overflow>`, `f(x: a, y: b)`, `conform i32: Zeroed`, `['r, 's]`, and `[10_u8, 20_u8]`.

Every nonempty physical line begins with exactly two ASCII spaces for each enclosing brace block. A closing brace is rendered after reducing the depth for the block it closes. A match-arm header is therefore one level inside its match, and statements in the arm body are two levels inside it.

The line-bearing simple productions are `field`, `variant`, `fn_sig`, `law`, `fn_bind`, `const_decl`, `doc`, `set_stmt`, `expr_stmt`, `return_stmt`, `break_stmt`, `check_stmt`, and `give_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs` or `propagate_let_rhs`. Each renders completely on one line, including its final semicolon.

The block-bearing productions are `struct_decl`, `enum_decl`, `contract_decl`, `conform_decl`, the body of `fn_decl`, `requires_block`, `loop_stmt`, `region_stmt`, `match_stmt`, `value_match`, and `arm`. Their introducer through `{` is one line; their children render on following lines at depth plus one; and `}` renders on its own line at the original depth. Empty blocks still use an opening line followed by a closing-brace line. A value-match let places its complete let prefix and the `match` introducer through `{` on one line.

A function without `requires_block` puts its complete header through the body `{` on one line. A function with `requires_block` puts its header through `requires {` on one line, renders the requires children, then renders the requires close and body open as the single line `} {`, followed by the body children and closing brace. Every production not listed as line-bearing or block-bearing introduces no formatting boundary of its own. Its terminals stay on the current line unless a descendant line-bearing or block-bearing production introduces one of the boundaries prescribed above. No other LF or blank line is emitted.

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*` excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)` (single token; the base has the raw lowercase-word shape used by IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a valid field-access place `p.field`: all five suffix words are reserved from field binding [OP-1, GRAM-5]; e.g. `iadd.checked`).

[FORM-4] There are no comments. Documentation is the `doc` field of declarations [GRAM-2]. Provenance lives in toolchain records.

[FORM-5] Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; a leading `-` is legal for signed TYPE, and the signed value must lie in TYPE's range [FORM-7]; e.g. `42_i32`, `-2147483648_i32`); finite floats use the grammar `-?(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE`, where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), positive exponents carry no sign, negative exponents carry one `-`, and only the integer and exponent components have the stated no-leading-zero form. Let C be the nonnegative integer formed by concatenating the integer and fraction digits, let F be the number of fraction digits, and let E be the signed integer formed by the exponent digits and their optional `-`; when the exponent is absent E is zero, and `e-0` also gives E zero. A matching decimal whose C is zero denotes signed decimal zero: a leading literal `-` selects negative zero and its absence selects positive zero, independently of E. Every other matching decimal denotes the exact nonzero rational whose magnitude is C × 10^(E − F), with the leading literal sign applied. For one finite bit pattern of TYPE, consider every matching decimal that rounds from that signed zero or exact nonzero rational to the bit pattern under IEEE 754 round-to-nearest, ties-to-even. Its canonical spelling is the candidate with the fewest ASCII bytes before `_TYPE`; a tie is resolved by lexicographically least unsigned ASCII bytes. This selection is total, host-independent, and unique; in particular `0.0` and `-0.0` remain distinct. Other examples are `1.5_f64` and `6.022e23_f64`. `unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, or one of exactly three escapes `\\ \" \n`; no other byte is legal, and each character has exactly one spelling (the escape where one is defined, the raw byte otherwise). STRING appears only in `doc` and `check` messages; non-ASCII diagnostic text is DEFERRED. There are no boolean literals: `Bool` is a prelude enum (§15). Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric contract (`Int` or `Float`, §15), denoting T's additive and multiplicative identity; a concrete type uses `0_i32` and the like, so there is no dual spelling. NaN and the infinities are not literals; they are the nullary ops `fnan` and `finf` [OP-1].

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual. The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling is the R3 one-spelling choice for the type's sole inhabitant.

[FORM-7] Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` is legal for signed T, and `-0` is written `0`. A float literal is legal only when it has the unique canonical spelling selected by [FORM-5] and denotes a finite value of its stated TYPE. An out-of-range integer, a leading-zero integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value.

[LEX-1] Lexicon policy: surface names label checked invariants, stated in this document self-containedly. Names are never borrowed from backend IR vocabulary (e.g. `noalias`), which names lowering consequences, not source invariants; and a name is borrowed from another language's convention only where a divergence census shows the semantics genuinely match. Ruling of record: the exclusive borrow mode is `uniq` (uniqueness-type lineage), not `mut` (Rust divergence: exclusivity is the invariant; mutation is only its permission, and the name breaks under future interior-mutability capabilities). DEFERRED with recorded delta: the two-axis mode vocabulary (exclusivity x write-permission, adding frozen/exclusive-read and capability-gated shared-write).

## 3. Grammar

[GRAM-1] The grammar is deterministic and unambiguous. Raw lexical formation scans each source independently from byte offset zero and partitions it into tokens and trivia without normalization, decoding a value, or consulting grammar position, name lookup, the operation table, or another source. At each cursor it takes exactly the following maximal form; no token or trivia crosses a source boundary.

- One or more ASCII space bytes form one trivia item. One LF byte forms one trivia item.
- A lower word starts with `[a-z]` and continues through the maximal `[a-z0-9_]*` suffix. If that complete base is followed immediately by `.` and exactly one of `wrap`, `trap`, `checked`, `sat`, or `strict`, and the suffix is not followed by an ASCII letter, ASCII digit, or `_`, the base, dot, and suffix instead form one operation-name token. Otherwise the lower word ends before the dot.
- An upper word starts with `[A-Z]` and continues through the maximal `[A-Za-z0-9]*` suffix.
- A region form starts with `'` and a label form starts with `@`; the sigil must be followed by `[a-z]`, after which the token continues through the maximal `[a-z0-9_]*` suffix.
- A numeric form starts with a decimal digit, or with `-` immediately followed by a decimal digit. It then consumes the maximal sequence of ASCII letters, ASCII digits, `_`, and `.`, plus a `+` or `-` only when that sign byte immediately follows `e` or `E`. Raw formation deliberately retains broad candidates such as `1e+`, `1.00_f64`, and `1.0E2_f64`; [FORM-5] and [FORM-7] decide membership and canonicality without rescanning or splitting them.
- A STRING form starts with `"` and ends at the first unescaped `"`. Its interior consists only of raw bytes `0x20` through `0x7e` other than `"` and `\`, or the two-byte escapes `\\`, `\"`, and `\n`. An escape consumes its backslash and follower together.
- `->` and `=>` are the two compound punctuation tokens. Otherwise each byte in `(`, `)`, `{`, `}`, `[`, `]`, `<`, `>`, `,`, `:`, `;`, `.`, `=`, and `&` is one exact punctuation token.

In source EBNF, each quoted fixed atom denotes the unique sequence of raw formed tokens whose concatenated bytes equal that atom. In particular, `"&uniq"` expands to the punctuation token `&` followed by the fixed lower-word token `uniq`, while `"->"` and `"=>"` each denote one compound punctuation token. The quoted `"[0-9]+"` atom in the `const` production is the sole pattern atom: it denotes one numeric-form token whose complete bytes match `[0-9]+`, and it is not a fixed atom. `SELECT_2` and the two-token parser bound count the expanded raw formed tokens, not quoted-atom occurrences. An external terminal denotes one predicate over one formed token.

Anything that cannot take one of those forms is a raw lexical defect with the attribution and exact span in [DIAG-1]. Raw formation gives every token exactly one context-free shape kind: lower word, upper word, region form, label form, operation-name form, numeric form, STRING form, or one exact punctuation form. Terminal membership then visits every formed token in source-ordinal and token order. For each token independently, and without consulting grammar position, name lookup, the operation table, or another token, it evaluates the complete approved set of exact fixed-terminal predicates and external-terminal predicates in this specification and retains every matching predicate. It rejects the token exactly when that retained set is empty; it never selects one preferred predicate and never tests only the predicates expected at a parser position. Grammar derivation later tests the retained predicate sets against its `SELECT_2` rows.

A grammar terminal is therefore a predicate over a token's shape kind and exact bytes, not a priority-selected replacement token kind. Exact-spelling and union predicates may overlap only when they do not compete at one grammar decision; every choice, optional, and repetition decision has pairwise-disjoint strong-LL(2) `SELECT_2` languages, so a parser selects exactly one arm with at most two tokens. In particular, a noncompeting overlap such as fixed `unit` with the `literal` union does not create an ambiguous parse, but no decision may use predicate priority to hide an overlap. Every production maps 1:1 to one core-tree node kind; there is no desugaring.

[GRAM-2] Items:

```
program      := item*
item         := fn_decl | struct_decl | enum_decl | contract_decl | conform_decl | const_decl
struct_decl  := "struct" TYPEID generics? "{" doc? field* "}"
field        := IDENT ":" type ";"
enum_decl    := "enum" TYPEID generics? "{" doc? variant* "}"
variant      := TYPEID "(" vfield_list? ")" ";"
vfield_list  := vfield ("," vfield)*
vfield       := IDENT ":" type
fn_decl      := program_kind? "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" rtype effects requires_block? "{" doc? stmt* "}"
program_kind := IDENT
requires_block:= "requires" "{" requires_entry* "}"
requires_entry:= doc | stmt
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

```
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

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | match_stmt
             | give_stmt
let_stmt    := "let" IDENT ":" mode type "="
               ( ordinary_let_rhs | propagate_let_rhs | value_match )
ordinary_let_rhs:= expr ";"
propagate_let_rhs := "propagate" expr ";"
set_stmt    := "set" place "=" expr ";"
expr_stmt   := call ";"
return_stmt := "return" expr ";"
loop_stmt   := "loop" LABEL "{" stmt* "}"
break_stmt  := "break" LABEL ";"
region_stmt := "region" REGIONID "{" stmt* "}"
check_stmt  := "check" expr "else" "trap" STRING ";"
give_stmt   := "give" expr ";"
match_stmt  := "match" expr "{" arm+ "}"
value_match := "match" expr "{" arm+ "}"
arm            := TYPEID "(" fieldbind_list? ")" "=>" "{" stmt* "}"
fieldbind_list := fieldbind ("," fieldbind)*
fieldbind      := IDENT ":" IDENT
```

[GRAM-5] Expressions and places:

```
expr           := atom | call | construct
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
               | "index" "<" type ">" "(" place "," atom ")"
psuffix        := "." IDENT
```

[GRAM-6] There is no operator syntax, no precedence, no infix, no `if`, no `while`, no `for`. Conditional control is `match` on prelude `Bool` [PRE-1]; a conditional value is a `let`-initializer `match` [GRAM-7, GIVE-1]; iteration is `loop` + `break`. `index` is a place (its sole home); bounds semantics are [OP-4].

[GRAM-7] `match` has one source arm shape (`{ stmt* }`, [GRAM-4]) and two distinct core-tree node kinds: `match_stmt` for a statement and `value_match` for a `let` initializer. They never compete at one grammar decision: a statement match begins at the statement boundary, while a value match begins only after the complete `let IDENT : mode type =` prefix. The parser therefore decides from source position alone, without type, name-resolution, or checker context. A `value_match` is value-producing, and every arm must satisfy the complete [GIVE-1] delivery judgment for its binding. A `match_stmt` produces no value; its arms act by effect and complete without one. `return`-position conditionals deliver by returning from arms; there is no helper-function conditional-initialization idiom, and value-production is confined to the `let` initializer, so a `match` never occupies an arbitrary expression position.

[GIVE-1] `give e;` delivers `e` as the value of the arm of the nearest enclosing `let`-initializer `match`; `e` must have that `let`'s declared `mode type` (stated at the binder [TYPE-5], never inferred from arms). `give` is legal only inside a `let`-initializer `match` arm — a checker-scoped restriction exactly as `break`'s enclosing-loop rule [TYPE-6]: the grammar admits `give_stmt` and the checker restricts it, so `give`'s legality, not its meaning, depends on the enclosing construct, which is META-2-clean by the `break` precedent. On every control path a `let`-initializer `match` arm terminates in exactly one `give e;` or cannot reach that value match's continuation; a give-free continuing path, a statement following a `give` in the same block, and a second `give` on one path are each a hard error citing GIVE-1 — the value analog of match exhaustiveness [ERR-2]. Give-completeness is a structural last-statement recursion: an arm delivers when its final statement is a `give_stmt`, a `return_stmt`, a `break_stmt` whose resolved target loop lexically encloses the same value match, or a `match_stmt` every arm of which delivers relative to that same value match. A final nested `value_match` delivers only to its own inner let and therefore does not make the outer arm deliver. A `check` or call that may trap also has a normally continuing edge and does not count as delivery or must-divergence. No `loop_stmt` is assumed to diverge. This recursion is strictly simpler than the ownership checker. `give e;` moves or copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions exactly as a returned borrow of the same mode [OWN-4].

[GRAM-8] Named construction. A `construct` of struct or enum-variant type K writes every declared field of K exactly once as `IDENT ":" atom`, the IDENTs equal to K's declared field names in declared order. A missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-8 and K's declared field list. There is no positional construction form; a nullary K is written `K()`. Field names are redundant-explicit facts (the TYPE-5 class): checked, never chosen, never a reordering option (declared order is the one legal byte sequence). The name-only-when-two-same-typed-fields alternative is a context-dependent spelling and is rejected [META-2].

[GRAM-9] Flat (three-address) computation. Every call argument, construct field value, and `index` offset is an `atom` [GRAM-5]; a `call` or `construct` in an atom position does not derive under the grammar and is a hard error citing GRAM-9. A computed value is forwarded to another operation only by binding it with a preceding `let` (stating its explicit mode and type [TYPE-5]) and referencing the binding. Nesting and let-splitting are not two spellings of one computation; there is no expression-nesting alternative [FORM-1]. `borrow_expr` is an `atom`, so borrows passed as arguments need no binding and OWN-6 is untouched.

[GRAM-10] Named match binders. An `arm` for variant K writes every declared field of K exactly once as `IDENT ":" IDENT` (the declared field name, then a fresh binder), in declared order; a missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-10 and K's declared field list. The binder is a fresh IDENT chosen by the writer and distinct from the field name, so TYPE-6 no-shadowing is never engaged by two arms binding fields of the same name. Binder modes remain derived by OWN-13 (not written). A nullary variant is written `K()`.

[GRAM-11] Named call arguments. A `call` whose callee resolves to a user `fn` or to an admitted system operation [SYS-1] writes its arguments as `fieldinit_list` [GRAM-5] — each `IDENT ":" atom` equal to the callee's declared parameter names in declared order, fixed by [FN-1] for a user `fn` and by [SYS-2] for a system operation, the GRAM-8 discipline applied to calls. A missing, extra, repeated, misspelled, or out-of-order parameter name is a hard error citing GRAM-11 and the callee's parameter list. A `call` whose callee resolves to a table operation [OP-1] writes positional `atom_list` operands (operands are order-intrinsic and unnamed). Argument reordering is not a spelling option: declared order is the one legal byte sequence [FORM-1], so parameter names are redundant checked facts (R4 anti-transposition), never a reordering license. Op-vs-fn is resolved by name lookup [OP-1], the same partition that already selects the callee.

## 4. Types

[CONST-1] The grammar production `const := "[0-9]+" | IDENT` is usable at `array<T, N>` sizes and `const` targs. A decimal integer literal is bare and u64 by position; an IDENT names an in-scope integer-typed const-generic parameter [GRAM-2] or a top-level integer-typed named-const item [CONST-2]. The set is closed and total: no operators, no calls, no in-language computation in v0. Constant-expressions are evaluated at monomorphization [FN-2]. An IDENT resolving to a non-integer or array-typed const is a compile-time rejection [DIAG-1]. This closes the const-generic forwarding path: `const N` is usable as an `array<T, N>` size and forwardable as a `const` targ. Const arithmetic is DEFERRED with recorded delta; when added it carries a distinct const-eval overflow-policy name, does not overload the runtime `.trap` OPNAMEs, and is excluded from EFF-2's exhibits-traps relation.

[CONST-2] A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value, with `cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]"`. `type` must be const-eligible: a primitive [TYPE-1], or `array<T, N>` of const-eligible T; `box`, `buffer`, `arena`, and `slice` are not const-eligible (a const is pure static rodata: no allocation, no region, no drop). The `cvalue` totally defines the value (T1): a primitive-typed const takes a FORM-5 numeric or unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T. The const-dependency graph is acyclic and declaration-before-use [TYPE-6]; evaluation is substitution and layout only. A const item is never `move`d, `set`, or `&uniq`-borrowed. It is read via `index`/`len` (copy-out for copy elements) or shared-borrowed `&'r p` in any region [OWN-10], so a const table may be `slice_of`-viewed and passed to a consumer. Struct/enum-typed consts are DEFERRED with recorded delta.

## 5. Ownership

[EFF-1] Row grammar: `effects := "pure" | effect ("," effect)*` with `effect := "reads" "(" REGIONID+ ")" | "writes" "(" REGIONID+ ")" | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "external" | "blocks" | "traps"`, in exactly this canonical order (reads, writes, allocates, external, blocks, traps). `pure` is the unique spelling of the empty row and therefore excludes `external` and `blocks` exactly as it excludes every other category. Frame residency (STOR-1) is not an allocation by definition. The two added categories take positions between `allocates` and `traps`, which leaves the pairwise canonical order of the four pre-existing categories unchanged.

A category states what a call may do, never which object it does it to. `external` states that the call may observe or change state outside ordinary Whitefoot memory, including file contents, cursors, output, host namespaces, clock and random sequences, resource lifetime, and compiler-derived resource release [STOR-3]. `blocks` states that an ordinary call may block its current host thread. Both are payload-free: neither takes a REGIONID, resource name, family name, or any other argument, and `external(cwd)`, `changes(file)`, and every other resource-parameterized effect spelling is outside this grammar and outside v0.18. A source row consequently carries no resource origin, and no rule derives a disjointness, reordering, or elimination conclusion from a row [EFF-5].

`external` and `blocks` are exact fixed grammar atoms and are therefore ineligible for IDENT under [FORM-3], like every other lowercase word this grammar fixes. The apostrophe- and at-prefixed lexical classes are untouched: REGIONID `'external` and LABEL `@blocks` remain well-formed spellings.

[EFF-2]
