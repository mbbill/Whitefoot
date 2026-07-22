# Kernel Specification v0.10

Status: REVIEW CANDIDATE v0.10 (2026-07-21; Phase-5 resolver entrance proposal). Proposes three grammar-selected TYPEID lookup domains, table-derived operation-name reservation, and deterministic declaration-inventory and lexical-resolution diagnostics. These bytes are non-authoritative review material until their complete evidence, protected-surface delta, full-document hash, exact owner approval, guarded baseline update, and active-target installation complete.

Prior: DRAFT v0.9 (2026-07-21; canonical-frontend entrance closure). Defines one executable tree-derived source format, a total host-independent finite-float spelling, exact raw-lexical and pre-tree diagnostic locations, ordered logical-source compilation-unit formation, and a deterministic terminal-predicate and strong-LL(2) grammar contract. The grammar excludes exact fixed lowercase terminals from IDENT, factors ordinary-let, try-let, value-match, and statement-match decisions, gives the two match positions distinct node kinds, admits literal law arguments, and parses semantic law-name and `requires` subsets as ordinary syntax before checking them. Source-law admission is fixed to one closed compiler-independent integer-table discharge rather than optional prover strength. This version also makes every top-level function signature visible throughout the closed compilation unit while retaining declaration-before-use for all other declarations, fixes the FORM-4 grammar reference, and assigns `requires` reservation rejection to FORM-3. These exact bytes are authoritative only after their complete evidence, protected-surface delta, and full-document hash receive advance owner approval and the bytes are installed through the guarded version-bump procedure.

Prior: DRAFT v0.8 (2026-07-19; tag-only enum equality). Adds `eeq` and `ene` as pure, total equality and inequality operations over one exact nominal tag-only enum type, including `Bool`; `ieq` and `ine` remain integer-only, and payload-enum equality, enum ordering, cross-enum comparison, and enum/integer conversion remain absent. Evidence and gates: optimizer-language-research/implementation/enum-equality-investigation/PACKET.md. Prior: DRAFT v0.7 (2026-07-18; bounded reborrow). Admits statement-scoped, non-escaping child reborrows: OWN-5 gains a child-creation carve-in and a suspended-holder carve-out, OWN-6 defines the statement-scoped child and its suspension/resumption, OWN-9 tightens the optimizer no-alias note to usable and non-suspended borrows, OWN-12 exempts the suspended ancestor from the effect-row check, new STOR-5 numbers the borrow-free and region-free storage invariant, and PATTERNS P4 becomes bounded reborrow; the deferred reference-result-provenance, uniq-to-shared downgrade, match-binder parents, grandchild chains, and bound children remain out. Evidence and conditions: optimizer-language-research/implementation/reborrow-investigation/PACKET.md. Prior: DRAFT v0.6 (2026-07-08; additive clusters). PROOF-2 first slice (owner-approved 2026-07-11) adds the checked, concrete-function-only `requires` prologue [FN-8]; the construct's existence and callee-entry semantics are evidence-selected, while its block spelling is R3-provisional pending writer/codegen comparison. Sub-step 1 (op-table completion) applied: integer bitwise/shift/rotate/bit-count/mulhi/saturating/min-max/abs, `reinterpret`, and the float math family (fneg/fabs/fmin/fmax/sqrt/floor/ceil/trunc/roundeven/frem/fma) added to OP-1, plus fgt/fge/fne and finf/fnan; OP-7 (op-name convention) and OP-8 (edge semantics + lowerings) added; OP-2/OP-3 restated; FORM-5 gains float-exponent and generic 0_T/1_T literals; Int/Float marker contracts added to the prelude. Sub-step 2 (data stack) applied: `buffer<T>` runtime-length heap value, `array_new`/`buffer_new` (OP-9 traps on size-overflow), `const` items (CONST-2) and a closed const-expr sublanguage (CONST-1 rewritten, re-adding the const-generic gparam that D8 deferred), collections ruled library/out-of-v0. Sub-step 3 (surface) applied: named-in-declared-order construction (positional removed, GRAM-8), three-address/ANF computation (GRAM-9), named match binders (GRAM-10), and the `give` value-match (GIVE-1, GRAM-7 rewritten) that deletes the helper-fn idiom; PRE-1 Option/Result payloads named; EX-1 re-cut. Polish (owner rulings 2026-07-08): named-in-declared-order arguments for user-fn calls (GRAM-11; no reordering, FORM-1), positional operands for table ops; global enum-variant-name uniqueness (TYPE-6); `reinterpret` extended to same-width int<->int resign (OP-1/OP-8); built-in Int/Float numeric conformance (FN-3); R0 confirmed to credit W3/W1 deltas (CONSTITUTION.md). Prior: DRAFT v0.5 (2026-07-07; Tier-0 errata). Fixed 11 internal self-contradictions (D1-D11): EX-1 is now parse/reprint/check-derivable; FORM-3 OPNAME tightened to a closed mode-suffix set with a GRAM-5 callee production (resolving the OPNAME/field-access collision); FORM-2 byte-format completed; FORM-7 (numeric-literal well-formedness), TYPE-7 (explicit deref typing), and OP-6 (cvt exact-or-Result, 29 total pairs, no rounding) added; DivError replaces DivideByZero; integer literals carry an inline sign (iK::MIN now writable); STRING pinned to ASCII-printable; the dead const-param gparam removed; META-1/META-4 dedup. Prior: DRAFT v0.4.1 (2026-07-07). DIAG-3 field schemas added (closes the audit-flagged R4-load-bearing deferral). GRAM-4 scrutinee widened to expr, resolving the GRAM-4/EX-1 contradiction found by M1 construction (bind-then-match rejected: per R3/W1 it taxes the sole conditional idiom with a mechanical temporary at every use); OWN-13 gains the owned-temporary clause. v0.3.1 added ERR-3 propagation (closed the R4-load-bearing deferral). v0.3 was the ledger-fix revision of v0.2 (META-6 derivation discipline; OP-2 ineg.wrap added with corrected rationale scope; FORM-6/FN-6/FN-7 derivation rationales recorded). v0.2 was the lexicon revision of v0.1 (owner rulings: borrow-mode rename mut->uniq; lexicon policy LEX-1). v0.1 was the revision of v0 under the round-1 spec critique (63 findings, 37 missing rules; all blocking and major findings addressed or explicitly deferred with recorded deltas). Section 5 (ownership) is FORMALLY RECONCILED against Featherweight Rust (spec/fr-reconciliation-m0.md: OBL-0..3 all discharged 2026-07-07, incl. verbatim paper pass with page-anchored quotes; the checker fragment is a proven sound subset of FR state space per T-A); the v0.7 bounded-reborrow edge is reconciled as a stricter-sound subset of FR reborrowing (checker state stays singleton-rooted; the lexical statement-scoped suspend is a subset of FR's flow prohibition), evidenced by a 1,000,000-program model-check with zero violations, with the verbatim FR *w-edge paper pass recorded as owed. Owner ratification word pending; no open technical obligations. Section 9 (effects) is gated on region/effect exemplar carding before ratification.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification — see decision-gates.md): loop form (GRAM-5/6), match-only conditionals and no-if (GRAM-6/PRE-1), statement-only match (GRAM-7), prefix arithmetic surface (OP-1/GRAM-6), interior annotation mandate (TYPE-5 — round-2 verdict still needs_evidence), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), deref/index prefix places (GRAM-5), and the `requires { requires_entry* }` surface spelling with its FN-8-checked ordinary-let/final-check subset (FN-8 — semantics selected, spelling not yet compared).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14). The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is accepted iff it parses under the canonical grammar, satisfies every rule in this document, and every unproven D1-critical checkable fact (bounds; alias-disjointness where a check form exists) carries a runtime check. There is no writer-emittable third state: nothing writer-stated is trusted unchecked. The sole trusted-assertion class is toolchain-gated ledger entries (§14), which the writer cannot author or edit.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code. This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.

[SCOPE-4] Contract violation at runtime traps: the process emits a machine-readable trap report (§12) and aborts. There is no unwinding.

## 2. Canonical form

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting. Non-canonical input is a hard error; the toolchain never auto-formats. Unknown constructs are hard errors (conservative extension).

[FORM-2] Each source file is UTF-8. Once every source has passed raw lexical formation and the complete compilation unit has one derivation, each source owns one ordered derivation forest: exactly the top-level `item` subtrees under the single compilation-unit `program` root whose terminals belong to that source, in source-local item order. A source forest is not a second `program` node, and a source with no items owns an empty forest. That source's canonical bytes are exactly the result of rendering its forest by the following rules. The input bytes must equal that rendering byte for byte; the toolchain does not normalize or rewrite input. A source that has no complete `item*` derivation is rejected by its owning lexical or grammar rule before this forest-format comparison, and no tree or forest is fabricated [DIAG-1].

Outside terminal interiors, lines end only with LF and formatting bytes are only ASCII space and LF. There is no CR, tab, trailing horizontal whitespace, leading blank line, or blank line inside a top-level item. A nonempty source has exactly one empty line between consecutive top-level `item` nodes and no trailing blank line; its final nonempty line ends with exactly one LF. A source containing zero items is exactly one LF. Terminal interiors retain their exact bytes and are checked by their owning FORM rule.

The left-attachment set contains `(`, `[`, `<`, `&`, and `.`. The right-attachment set contains `)`, `]`, `>`, `,`, `;`, `.`, `:`, `(`, and `<`. Between two consecutive terminals on the same line, emit zero bytes when the left terminal is in the left-attachment set or the right terminal is in the right-attachment set; otherwise emit exactly one ASCII space. Thus function headers are `fn f()`, `fn f<T>()`, and `fn f ['r]()`; generic and square-bracket interiors are compact; `](` and `>(` are attached; and commas and colons attach to their left operand and have one space before the grammar-required following element. Examples include `Result<i32, Overflow>`, `f(x: a, y: b)`, `conform i32: Zeroed`, `['r, 's]`, and `[10_u8, 20_u8]`.

Every nonempty physical line begins with exactly two ASCII spaces for each enclosing brace block. A closing brace is rendered after reducing the depth for the block it closes. A match-arm header is therefore one level inside its match, and statements in the arm body are two levels inside it.

The line-bearing simple productions are `field`, `variant`, `fn_sig`, `law`, `fn_bind`, `const_decl`, `doc`, `set_stmt`, `expr_stmt`, `return_stmt`, `break_stmt`, `check_stmt`, and `give_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs` or `try_let_rhs`. Each renders completely on one line, including its final semicolon.

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
fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" rtype effects requires_block? "{" doc? stmt* "}"
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
param        := IDENT ":" mode type
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
               ( ordinary_let_rhs | try_let_rhs | value_match )
ordinary_let_rhs:= expr ";"
try_let_rhs := "try" expr ";"
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

[GRAM-11] Named call arguments. A `call` whose callee resolves to a user `fn` writes its arguments as `fieldinit_list` [GRAM-5] — each `IDENT ":" atom` equal to the callee's declared parameter names in declared order [FN-1], the GRAM-8 discipline applied to calls. A missing, extra, repeated, misspelled, or out-of-order parameter name is a hard error citing GRAM-11 and the callee's parameter list. A `call` whose callee resolves to a table operation [OP-1] writes positional `atom_list` operands (operands are order-intrinsic and unnamed). Argument reordering is not a spelling option: declared order is the one legal byte sequence [FORM-1], so parameter names are redundant checked facts (R4 anti-transposition), never a reordering license. Op-vs-fn is resolved by name lookup [OP-1], the same partition that already selects the callee.

## 4. Types

[TYPE-1] Primitive types: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 unit`. (`Bool` is a prelude enum, §15, not a primitive.)

[TYPE-2] Composite types: `struct`, `enum`, `array<T, N>` (N a constant-expression, [CONST-1]), `slice<'r, T>` (region-carrying view), `box<T>` (heap-owned unique), `arena<'r, T>` (region-bounded owned), `buffer<T>` (heap-owned, runtime-length, flat contiguous {data-pointer, u64 length} value; affine single-owner; length fixed at allocation, no in-place growth). v0 buffer/array element type T must be copy (a primitive or tag-only enum, per the OWN-1 copy amendment); affine-element buffers are DEFERRED with recorded delta (blocked on the §5 take/replace resolution).

[TYPE-3] Nameability: every constructible type/mode/effect has a canonical, finite, writable name requiring no compiler execution.

[TYPE-4] There are no implicit conversions. Representation change is the single explicit op `cvt<Src, Dst>(x)`. Totality is decided by value-preservation, not bit-width: `cvt` returns `own Dst` where every value of Src is exactly representable in Dst, and `own Result<Dst, NarrowError>` for every other distinct numeric pair; it never rounds, truncates, or saturates. The exact partition and per-value semantics are [OP-6]. Deliberate rounding is a separate DEFERRED float-round op family, never `cvt`.

[TYPE-5] No inference across statements: every `let` states its full mode and type; call sites state all type/region/const arguments explicitly; argument types match declared parameter types exactly.

[TYPE-6] Name resolution uses the following closed declaration domains. The grammar role, never an inferred type or expected result, selects the domain and admissible declaration class.

| domain | declarations | admitted uses |
|---|---|---|
| lexical IDENT | top-level `fn_decl`; top-level `const_decl`; const `gparam`; `param`; `let_stmt`; second IDENT of `fieldbind` | a `callee` or `fn_bind` right IDENT admits only a top-level function; `const` IDENT admits only an in-scope const generic or earlier named const; `cvalue` IDENT admits only an earlier named const; `pbase` admits only an in-scope value binding or named const |
| nominal-type TYPEID | source `struct_decl` and `enum_decl` names; PRE-1 nominal types; lexical type `gparam`s overlay this domain while live | `type` TYPEID and the TYPEID suffix of a FORM-5 generic numeric literal admit a live type generic where that form requires one, otherwise a nominal type |
| constructor TYPEID | each source struct constructor under its struct TYPEID; every source enum `variant`; PRE-1 variants, classified as struct-constructor or enum-variant | the leading TYPEID of `construct` admits either class; the leading TYPEID of `arm` admits only enum-variant |
| contract TYPEID | source `contract_decl` names and PRE-1 contract names, including `Int` and `Float` | the optional bound TYPEID of a type `gparam` and the contract TYPEID of `conform_decl` |
| REGIONID | `region_params` and `region_stmt` | every REGIONID in `type`, `mode`, `targ`, `effect`, and `borrow_expr` |
| LABEL | `loop_stmt` | `break_stmt` |

A source struct contributes one declaration event that adds one nominal-type entry and one constructor entry with the same spelling. Those entries do not collide because the grammar distinguishes a `type` role from a `construct` or `arm` role. An enum declaration adds only its nominal type; each variant adds its constructor. Entries must be unique within, but not across, the nominal-type, constructor, and contract domains. Constructor uniqueness is whole-unit and context-free, so construction and matching never consult an expected nominal type.

PRE-1 contributes exactly twenty-four declaration records in this preorder: each enum nominal, then its type parameters in list order, then each variant and that variant's fields in list order, followed by the contracts in declaration order. They are six nominal enums, ten enum-variant constructors, three owner-local type parameters (`Option.T`, `Result.T`, and `Result.E`), three owner-table fields, and two contracts. Exactly the six nominals, ten constructors, and two contracts enter the source resolver's whole-unit lookup inventory and are visible throughout the closed unit. The three type parameters resolve only within their owning compiled PRE-1 declaration, the three fields enter only their owning variant table, and none of those six owner-local records is visible to source lookup. PRE-1 records have no source event or source node. Every top-level function signature is visible throughout the closed compilation unit after unit formation and before any semantic use is resolved [FN-1]. A source nominal type or contract becomes visible immediately after its declaring TYPEID terminal. A source struct constructor becomes visible at that same terminal; an enum-variant constructor becomes visible immediately after its variant TYPEID terminal. Each remains visible through the end of the unit. Whole-unit inventory checks uniqueness but grants no earlier visibility; a use before one of these declaration points is rejected even though inventory knows the later declaration exists.

A generic TYPEID parameter becomes visible after its declaring terminal through the remainder of its declaration's generic, header, and body scope. It may not redeclare another parameter in the same generic list or shadow a live nominal type or enclosing generic type. Constructor and contract spellings are separate grammar-selected domains and do not participate in that comparison. A const generic becomes visible after its complete `gparam`. A region parameter becomes visible after its terminal through the remainder of its signature and body; for `fn_sig`, that scope ends at the signature terminator. Independently of visibility, OWN-3 requires every REGIONID declaration to be unique throughout its owning function declaration or contract-member signature, parameters included: a later region parameter or local region may not reuse an earlier region spelling even after the earlier region's lexical scope has ended. A `fn_decl` parameter becomes visible after its complete `param` through the function's requires block and body. A `fn_sig` parameter becomes visible after its complete `param` through that signature's terminator; duplicate parameters in that signature are same-scope redeclarations even though v0.10 has no lexical value-use role in the remaining suffix. A `let_stmt` binder becomes visible only after its complete initializer statement through the end of its lexical block; a requires-block let is visible only to later requires entries and not to the function body [FN-8]. A match binder becomes visible in its arm body only after the complete fieldbind list and only after GRAM-10 has established that it differs from its paired field label, every earlier binder in that arm list, and every lexical-IDENT declaration live on arm entry. A loop label and local region are visible only in their respective bodies. A named const becomes visible only after its complete `const_decl`, preserving CONST-2's explicitly-earlier rule.

Within one domain, two declarations in the compilation-unit root or in the same lexical scope are a redeclaration attributed to the later declaration event. Declarations in unrelated function or declaration owners are not duplicates merely because their spellings match. A nested lexical declaration may not shadow an entry live at that declaration. OWN-3's function-wide REGIONID uniqueness is stricter than either rule and is reported at the later region declaration with the conflicting region origin. GRAM-10 exclusively owns match-binder distinctness and freshness: a second `fieldbind` IDENT equal to its paired field label, an earlier binder in the same arm list, or any lexical-IDENT declaration live on arm entry is rejected citing GRAM-10 at that later/offending binder before it becomes a declaration, rather than also being reported as TYPE-6 shadowing. Because every top-level function is live throughout the unit, any other parameter, local, or const generic in a nested scope may not use a top-level function spelling even when that function's source item occurs later; the nested declaration is the offending shadow event. Disjoint expired lexical scopes may reuse an ordinary value or label spelling; REGIONID reuse remains forbidden throughout one function by OWN-3. Logical paths and record boundaries never create a namespace, scope, or lookup key [PROG-2].

The owner-dependent declaration and use roles are exactly the carriers classified by [DIAG-1]. They do not enter or query a lexical name domain. DIAG-1 retains each for later typed owner/member checking. Deferral is neither acceptance nor rejection of its later owner/member relation.

[TYPE-7] Reading through a reference is explicit. `deref(place)` where place has type `&'r T`, `&uniq 'r T`, `box<T>`, or `arena<'r, T>` denotes a place of referent type T [GRAM-5]; a use of that place copies it when T is copy and requires `move` when T is affine [OWN-1]. A borrow-mode or box/arena binding used where a value of its referent type T is expected is a hard error citing TYPE-7, with the mechanical fix `deref(.)`. There is no implicit read-through-borrow [TYPE-4, META-2].

[CONST-1] The grammar production `const := "[0-9]+" | IDENT` is usable at `array<T, N>` sizes and `const` targs. A decimal integer literal is bare and u64 by position; an IDENT names an in-scope integer-typed const-generic parameter [GRAM-2] or a top-level integer-typed named-const item [CONST-2]. The set is closed and total: no operators, no calls, no in-language computation in v0. Constant-expressions are evaluated at monomorphization [FN-2]. An IDENT resolving to a non-integer or array-typed const is a compile-time rejection [DIAG-1]. This closes the const-generic forwarding path: `const N` is usable as an `array<T, N>` size and forwardable as a `const` targ. Const arithmetic is DEFERRED with recorded delta; when added it carries a distinct const-eval overflow-policy name, does not overload the runtime `.trap` OPNAMEs, and is excluded from EFF-2's exhibits-traps relation.

[CONST-2] A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value, with `cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]"`. `type` must be const-eligible: a primitive [TYPE-1], or `array<T, N>` of const-eligible T; `box`, `buffer`, `arena`, and `slice` are not const-eligible (a const is pure static rodata: no allocation, no region, no drop). The `cvalue` totally defines the value (T1): a primitive-typed const takes a FORM-5 numeric or unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T. The const-dependency graph is acyclic and declaration-before-use [TYPE-6]; evaluation is substitution and layout only. A const item is never `move`d, `set`, or `&uniq`-borrowed. It is read via `index`/`len` (copy-out for copy elements) or shared-borrowed `&'r p` in any region [OWN-10], so a const table may be `slice_of`-viewed and passed to a consumer. Struct/enum-typed consts are DEFERRED with recorded delta.

## 5. Ownership, regions, borrows (PROVISIONAL pending formal-calculus reconciliation)

[OWN-1] Every value has exactly one owner. Values are classified copy or affine: primitives (TYPE-1), shared borrows, and tag-only enums (every variant nullary; `Bool` is the canonical case) copy on use; all other values (owned composites, `box`, `arena`, `slice` as `&uniq`, uniq borrows) are affine. An affine value is consumed by `move p` exactly once; a bare `place` expression of affine type is a hard error (write `move p`), and `move p` on a copy value is a hard error (copy values are used bare — one spelling per meaning, FORM-1). After a move, the whole binding rooting `p` is dead (partial moves kill the whole binding); any later use, write, or `set` of a dead binding is an error — reinitialization requires a new `let`.

[OWN-2] Modes: `own` (owned), `&'r` (shared borrow in region `'r`), `&uniq 'r` (exclusive borrow in region `'r`). Modes are always written.

[OWN-3] Regions are lexical. `region 'r { ... }` introduces `'r`; `region_params` introduce caller-supplied regions. Region identifiers are unique within a function (parameters included). Outlives-or-equals is the total reflexive relation: `'a` outlives-or-equals `'b` iff `'a = 'b`, or `'a`'s block strictly encloses `'b`'s block, or `'a` is caller-supplied and `'b` is local. Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject).

[OWN-4] A borrow `&'a p` / `&uniq 'a p` is live exactly until the end of `'a`'s block (named-region liveness). It may be stored into a destination of declared region `'b`, passed to a parameter of region `'b`, or returned as `rtype` region `'b`, only if `'a` outlives-or-equals `'b`.

[OWN-5] Resolved-place exclusivity. While `&uniq 'a p` is live and its holder is not suspended [OWN-6]: no place overlapping resolved(`p`) may be read, written, moved, or borrowed, except reads/writes through that borrow's holder and except the creation of a statement-scoped child reborrow of that holder [OWN-6]. While a holder is suspended (a live child reborrow of it exists), its own read/write allowance is withdrawn: no read, write, move, copy, or call-transfer through it is admitted until its last child ends. While any `&'a p` is live: no place overlapping resolved(`p`) may be written, moved, or uniq-borrowed; reads are permitted. Content reached through any borrow may never be moved: `move` requires a place rooted at an own-mode binding. Exclusivity invariant, checked unconditionally: no two live usable `&uniq` borrows have overlapping resolved places; a suspended holder is not usable, so the only overlapping pair, a suspended parent and its child, is never both-usable by construction.

[OWN-6] Holder, resolution, and statement-scoped child reborrow. The holder of a borrow is the binding its `borrow_expr` initializes (a borrow not bound by `let` is a call-scoped temporary, live until the end of the enclosing statement). resolved(place) rewrites a place rooted at a holder binding to the borrowed place plus the appended suffix, recursively. All OWN-5/OWN-7 judgments use resolved places. A statement-scoped child reborrow is the written form `&uniq 'c deref(h)[suffix]` or `&'c deref(h)[suffix]` occurring as an argument atom of a `call` expression [GRAM-9], admitted only when: the receiving call's result mode is `own` or `unit`, never a borrow; `'c` is a locally-introduced region [OWN-3] whose block does not extend beyond the enclosing statement, and a caller-supplied region parameter is not admitted; the eligible holder `h` is a function parameter or a `let`-bound borrow, never a `match` binder; and a `uniq` child has a `uniq` parent, while a `shared` child is admitted from either [OWN-5]. resolved(child) = resolved(`h`) ++ suffix. Creating a child suspends `h` for the enclosing statement [OWN-5]; while suspended, the sole operation admitted through a place overlapping resolved(`h`) is creating a further sibling child, siblings judged by OWN-7 with any overlapping pair containing a `uniq` child an error, and `h` resumes at the end of the statement after its last child ends. A child is never bound, returned, `give`n, stored, or the whole call result, and its `'c` cannot outlive the statement, so no borrow derived from a child outlives its statement; with borrow-free storage [STOR-5] the child is non-escaping. Bound children, result-carrying children (reference-result provenance), `uniq`-to-`shared` downgrade, `match`-binder parents, and grandchild reborrow chains are DEFERRED with recorded delta.

[OWN-7] Overlap: resolved `p` overlaps resolved `q` iff one is a prefix of the other. Two `index` places with the same resolved base overlap iff their indices are not both literals with unequal values. Two `slice` values over the same resolved root overlap conservatively.

[OWN-8] Reject-when-unsure: the checker rejects any program it cannot prove conformant. Rejection of a sound-but-unprovable program is not a defect; the diagnostic names the rule and a restructuring.

[OWN-9] Non-normative consequence for the optimizer: a live, usable `&uniq` borrow's resolved place is unaliased by any other usable access path (a suspended holder [OWN-6] is not usable; a statement-scoped child and its suspended ancestor, though both live, are never mutually noalias — the guarantee is one usable mutable path per place [OWN-5]); shared borrows are read-only for their duration; owned values are unaliased except by their own live shared borrows.

[OWN-10] Borrow-storage duration: `&'a p` is legal only if `p`'s storage outlives `'a`. For `p` rooted at an own-mode binding b: `'a` must be introduced within b's scope (never a caller-supplied region, for locals and own parameters alike). For `p` rooted at a borrow of region `'b`: `'b` must outlive-or-equals `'a`. For `p` rooted in `arena<'r, T>` content: `'r` must outlive-or-equals `'a`. For `p` rooted at a named `const` item [CONST-2]: any region `'a` is legal; immutable static storage has program lifetime and outlives every region.

[OWN-11] Loops: inside `loop @l`, a `borrow_expr` may only name regions introduced inside `@l`'s body; bindings declared outside `@l` may not be moved inside it (copies exempt).

[OWN-12] Calls (OWN-CALL cluster): at a call, declared region parameters are substituted with the caller's region arguments, which must be live; argument borrows are live accesses of their resolved places for the duration of the call and are judged under OWN-5 (two `&uniq` arguments whose resolved places overlap are an error); the callee's effect row, instantiated at the actual regions, is checked against the caller's live borrows under OWN-5. When an argument is a statement-scoped child reborrow [OWN-6], its suspended ancestor holder is excluded from this effect-row overlap check, since the child, not the ancestor, holds the claim for the call; every non-ancestor live borrow is still checked.

[OWN-13] Match ownership: a non-place expression scrutinee is an owned temporary (moved into the match). Matching a place of own mode moves it (the binding dies; binders receive `own` payloads); matching through `&'r` / `&uniq 'r` leaves the scrutinee live and binds payloads as `&'r` / `&uniq 'r` respectively. Binder modes are derived by this rule, stated once; they are not written. A `let`-initializer `match` binds its value from arm `give`s [GIVE-1]; scrutinee treatment and binder-mode derivation are unchanged, and each arm delivers a value of the `let`'s declared mode and type, so on the taken arm an `own` result is moved exactly once (no double-move; T1 preserved). A `give e;` whose `e` is a borrow reaching through a binder or an outer borrow obeys [OWN-4]/[OWN-5] exactly as a returned borrow of the same mode. This arm-result region join is an additive reuse of the return-of-borrow judgment and is PROVISIONAL pending confirmation against the formalized calculus before section-5 ratification (D1a).

## 6. Storage

[STOR-1] Storage class is a function of type, stated once: `box<T>` is heap-owned; `arena<'r, T>` is arena-owned, bounded by `'r`; `buffer<T>` is heap-owned (one compiler-derived heap allocation, released by one compiler-derived free at owner scope-exit [STOR-3]); a `const` item [CONST-2] is immutable static storage (program-lifetime, read-only, never dropped); every other owned value is frame-resident (inline in its owner or the stack frame). There is no per-binding storage annotation and no default clause. The reserved storage-contract field `foreign_shared` exists in the vocabulary but is legal only in programs containing gated FFI frames (§14); compiler-inferred demotion of an allocation to foreign-shared is a floor violation. Growable or keyed collections (dynamic vector, hash map, set, byte-string, text) are neither storage classes nor kernel constructs: they are future library structures over `buffer<T>` plus struct/enum and generics (a byte-string is `buffer<u8>`; a growable vector pairs a `buffer<T>` with a length). They are out-of-v0-kernel and recorded, additionally blocked on the §5 take/replace resolution that in-place buffer replacement requires; the arena-index-pool ownership pattern is rejected as a collection basis (it resurrects use-after-free as well-typed slot-recycling). Char and Unicode text are out-of-v0, recorded.

[STOR-2] Creation: `box_new<T>(v)` returns `own box<T>`; `arena_new<'r, T>(v)` returns `own arena<'r, T>`; both are ordinary calls in the operation table. Content access is through `deref`.

[STOR-3] Deallocation is compiler-derived and artifact-surfaced: every drop and arena release appears as an explicit operation in the elaborated artifact. Every control-flow edge leaving a region block (fallthrough, `break`, `return`) carries that region's releases and drops, in reverse declaration order. No finalizers; no reference counting. A `buffer<T>` drop is a compiler-derived heap free, surfaced like a `box<T>` drop on every region-exit edge in reverse declaration order; `const` items [CONST-2] are never dropped.

[STOR-4] Arena confinement: a value of type `arena<'r, T>` may not be returned, stored into a field, or moved to a destination outside `'r`'s block; borrows of its content obey OWN-10 with source region `'r`.

[STOR-5] Storage is borrow-free and region-free. No struct field, enum variant payload, `array`/`buffer` element, or `box`/`arena` content may be a borrow: the `field`/`vfield` grammar admits only `type` [GRAM-3], and `type` has no borrow (`&` / `&uniq`) production. No `struct` or `enum` declaration is parameterized by a region (`generics`/`gparam` admit no REGIONID [GRAM-3]), so no stored field may name a caller region — in particular no field may be `slice<'r, T>` or `arena<'r, T>`, whose region has no declaration site to bind it. Consequently a borrow is structurally unstorable, and a borrow can leave a callee only through its return value; this is the storage half of the statement-scoped child reborrow's non-escape guarantee [OWN-6].

## 7. Operations

[OP-1] Every computation is a call naming one operation from the operation table; one operation per (semantic operation × mode); nothing is overloaded. The table below is the normative inventory (columns: op, type domain, signature, effects).

| op | domain | signature | effects |
|---|---|---|---|
| `iadd.wrap` `isub.wrap` `imul.wrap` | all int T | `(T, T) -> own T` | pure |
| `iadd.trap` `isub.trap` `imul.trap` | all int T | `(T, T) -> own T` | traps |
| `iadd.checked` `isub.checked` `imul.checked` | all int T | `(T, T) -> own Result<T, Overflow>` | pure |
| `idiv.trap` `irem.trap` | all int T | `(T, T) -> own T` | traps |
| `idiv.checked` `irem.checked` | all int T | `(T, T) -> own Result<T, DivError>` | pure |
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
| `iand` `ior` `ixor` | all int T | `(T, T) -> own T` | pure |
| `inot` | all int T | `(T) -> own T` | pure |
| `ishl.wrap` `ishr.wrap` | all int T | `(T, u32) -> own T` | pure |
| `ishl.trap` `ishr.trap` | all int T | `(T, u32) -> own T` | traps |
| `irotl` `irotr` | all int T | `(T, u32) -> own T` | pure |
| `ipopcount` `iclz` `ictz` | all int T | `(T) -> own u32` | pure |
| `ibswap` | int T, width>=16 | `(T) -> own T` | pure |
| `imulhi` | all int T | `(T, T) -> own T` | pure |
| `iadd.sat` `isub.sat` `imul.sat` | all int T | `(T, T) -> own T` | pure |
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

Let `DotlessOperationNames` be exactly the set of distinct individual operation spellings enumerated in this rule's normative `op` column whose complete spelling satisfies IDENT and contains no dot. Let `ModeWords` be exactly the suffix alternatives in FORM-3's active OPNAME formation rule; in this version it equals `{wrap, trap, checked, sat, strict}`. `ReservedLowerNames` is exactly `DotlessOperationNames` union `ModeWords`. A printed review list is non-authoritative and, when present, must equal the corresponding derived set.

Each distinct complete spelling in the operation table declares one operation-family identity, even when more than one row carries that spelling; the two `cvt` rows therefore belong to one `cvt` family. An OPNAME callee resolves to its exactly spelled operation family. An IDENT callee whose spelling belongs to `DotlessOperationNames` resolves to that operation family; every other IDENT callee admits only a top-level source `fn_decl`. Absence from the selected operation-family or function inventory is a hard error citing OP-1. Later typed operation checking uses the written type arguments and operand domains to select the applicable row within the resolved family. Operand types never select between an operation family and a function.

No source declaration in this closed list may use a member of `ReservedLowerNames`: the IDENT of `fn_decl`; the IDENT of `const_decl`; every `param` IDENT; every `let_stmt` IDENT, including ordinary, try, value-match, and requires-block lets; the second IDENT of `fieldbind`; every `field` and `vfield` IDENT; and the IDENT-shaped interior of `region_params` and `region_stmt`. Such a reserved binding is rejected citing exactly FORM-3. Dependent field declarations participate in this pre-resolution reservation inventory even though their owner/member duplicates remain deferred. No other declaration role is covered: type-generic TYPEIDs, const-generic IDENTs, LABELs, and contract-member `fn_sig` IDENTs remain outside this prohibition. Dotted OPNAMEs cannot be declarations under the grammar. This reservation keeps operation-versus-function resolution context-free [META-2] and keeps a field-access place from maximal-munching as OPNAME [FORM-3].

[OP-2] There are no wrap modes for division/remainder because no sound modular semantics exists for divisor-zero; this is table data, not an exception clause. (Negation has a wrap mode: two's-complement wrapping negation is sound modular arithmetic — ledger fix 2026-07-07.) Integer division and remainder have two checkable failures: a zero divisor for all int T, and, for signed T, the single signed-overflow case `iK::MIN / -1` (LLVM sdiv/srem are UB on both); `.trap` traps on either, and `.checked` returns `Err(DivideByZero())` for a zero divisor and `Err(DivOverflow())` for signed `iK::MIN / -1`, else `Ok`. DivOverflow is statically unreachable for unsigned T; the uniform `DivError` type is retained for regularity. Both classifications are table-fixed [ERR-4]. Mode-axis membership per family is table data: add/sub/mul carry {wrap, trap, checked, sat}; div/rem carry {trap, checked}; ineg and iabs carry {wrap, trap, checked}; shifts carry {wrap, trap}. Masking a shift amount discards writer intent, so a trap rung is offered; masking a rotate amount is the exact identity, so rotates are dotless-total [OP-8].

[OP-3] Float ops that ROUND carry `.strict` (IEEE 754, no reassociation, no contraction) and are the family a future fast-math mode would relax: `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` `fsqrt.strict` `ffma.strict`. Float ops that are EXACT or exact-selection are dotless: `fneg` `fabs` `fcopysign` `fmin` `fmax` `ffloor` `fceil` `ftrunc` `froundeven` `frem` and the six comparisons. Approximation/fast-math modes remain an OPEN numeric-semantics question; a relaxed float op would be introduced as a distinct OPNAME (FORM-1-additive).

[OP-4] `index<T>(p, i)` reads/writes are bounds-checked in all build modes when unproven; out-of-bounds traps [SCOPE-4]. "Proof" means deterministic-checker or verified-proof-artifact discharge; a solver may only promote performance-ledger facts and never licenses check elision. `index` applies to `array<T, N>`, `slice<'r, T>`, and `buffer<T>` places; a `buffer<T>` index is bounds-checked against the runtime length.

[OP-5] `check e else trap "msg";` is a runtime check in all build modes, never elided. A passed check creates the checked fact on the dominated path (stated-and-checked channel); the fuller stated-and-checked vocabulary (loop invariants, ranges) is DEFERRED with its delta.

[OP-6] cvt partition and semantics (cross-reference TYPE-4). `cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value is exactly representable in Dst (y the unique such Dst value) and `Err(NarrowError())` otherwise, and it never rounds, truncates, or saturates. A non-integral float-to-int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or infinity targeting an integer all yield `Err`; for float-to-float, an infinity maps to the same infinity and NaN maps to the target canonical quiet NaN (value-preserving). A pair is TOTAL — signature `(Src) -> own Dst`, no Result — where every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`. Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.

[OP-7] Operation-name convention (regularity, W1-predictable). An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix. `Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family. A `.mode` suffix appears iff the op sits on a mode axis, and single-behavior ops are dotless; the mode axes are the integer result-overflow axis {wrap, trap, checked, sat}, the shift out-of-range-amount axis {wrap, trap}, and the float rounding axis {strict}, with per-family membership fixed by [OP-2]. Signedness-parametric lowering keyed on the explicit type argument (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `ilt` = `slt`/`ult` row, not overloading. Nominal enum identity is likewise checked from the explicit type argument before `eeq`/`ene` lowering; equal representation width never makes distinct enum types interchangeable.

[OP-8] Edge semantics and confirmed lowerings for the operations added in this revision; every totality edge is closed here as table data, so no added row is writer-reachable poison (per T2 and W3). `iand`/`ior`/`ixor` lower to `and`/`or`/`xor` and `inot` to `xor x, -1` (total). A shift or rotate amount is `u32`; `ishl.wrap`/`ishr.wrap` mask the amount to `amt & (width-1)` and are total, `ishl.trap`/`ishr.trap` trap when `amt >= width`, `ishr` is `ashr` for signed T and `lshr` for unsigned T, and `irotl`/`irotr` lower to `llvm.fshl`/`llvm.fshr` whose amount is taken modulo width, so rotates are total. `ipopcount` is `llvm.ctpop`; `iclz`/`ictz` are `llvm.ctlz`/`llvm.cttz` with is-zero-poison false, so a zero input returns the bit width (the zero-input fix); counts return `u32`. `ibswap` is `llvm.bswap` (width a multiple of 16). `imulhi` is the high half of the full double-width product. `iadd.sat`/`isub.sat` are `llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat` clamping to T's range; `imul.sat` widens, multiplies, and clamps, which avoids the signed-saturation miscompile in `llvm.smul.fix.sat`. `imin`/`imax` are `llvm.smin`/`umin` or `smax`/`umax`. `iabs.wrap`/`.trap`/`.checked` use `llvm.abs` with is-int-min-poison false, so `abs(iK::MIN)` is `iK::MIN` (the defined two's-complement edge value): `.wrap` returns it, `.trap` traps on it, and `.checked` returns `Err(Overflow())`. `reinterpret` is the LLVM bitcast instruction for cross-domain pairs (int<->float; bit-preserving, all NaN payloads and sign bits preserved) and an identity bit-relabel for same-width int<->int resign (i8<->u8, i16<->u16, i32<->u32, i64<->u64); it is the bit-preserving counterpart of value-preserving `cvt`, giving bit-level resign a home distinct from cvt's value-preserving resign. `fneg` is the LLVM fneg instruction (a sign-bit flip, not `fsub(0.0, x)`); `fabs` is `llvm.fabs`; `fcopysign` is `llvm.copysign`. `fmin`/`fmax` are `llvm.minimum`/`llvm.maximum` (IEEE-2019, NaN-propagating, negative zero ordered below positive zero, deterministic); `llvm.minnum`/`maxnum` are not used, because their signed-zero tie result is unspecified and breaks the reproducibility FORM-1 requires. `ffloor`/`fceil`/`ftrunc` are `llvm.floor`/`ceil`/`trunc` (roundToIntegral, staying in the float type); `froundeven` is `llvm.roundeven` (ties-to-even, matching `fadd.strict`). `frem` is the LLVM frem instruction (the C `fmod`: remainder with the dividend's sign, truncated quotient, exact), a distinct operation from IEEE `remainder`. `fsqrt.strict` is `llvm.sqrt` and `ffma.strict` is `llvm.fma` (single-rounding fused, distinct from the contraction [OP-3] forbids; a correctly-rounded libcall on hardware without an FMA unit). The comparisons `feq`/`flt`/`fle`/`fgt`/`fge` are ordered (`fcmp o*`, false when either operand is NaN) and `fne` is unordered (`fcmp une`), so `fne` equals `bnot(feq)` on every input and `fne(x, x)` is true exactly when x is NaN. `finf` is the positive-infinity value (negative infinity is `fneg(finf<T>())`) and `fnan` is the canonical quiet NaN; other NaN payloads are reachable through `reinterpret`. For a tag-only enum T, `eeq<T>(a, b)` is `True()` exactly when `a` and `b` denote the same declared variant of the same nominal T, and `ene<T>(a, b)` is its exact boolean complement. Both operands and the explicit type argument must have that exact T; representation equality never permits cross-enum comparison. `Bool` is admitted by the same tag-only rule. Both operations lower directly to equality or inequality of the validated discriminants in T's already-selected representation. They are pure and total: after normal operand evaluation, the primitive does not inspect a payload, access memory, trap, convert a value, or introduce a new optimizer fact channel; an operand read still exhibits its ordinary effect before the primitive executes. Payload-carrying enums, enum ordering, and enum/integer conversion remain outside the operation table.

[OP-9] `buffer_new<T>(n, v)` computes its allocation byte-size as `n * sizeof(T)` in u64 (sizeof(T) is a monomorphization-time constant). When this product overflows u64, `buffer_new` traps [SCOPE-4] before allocating: an unrepresentable buffer size is a contract violation, never a silent under-allocation (R4: no silent corruption; T2: no-UB), so `buffer_new`'s effect row includes `traps`. This is the sole allocation-size hazard `box_new`/`arena_new` (single-T, no runtime multiply) do not have. Allocation failure (OOM) is handled as by `box_new` (TCB-level, SCOPE-3), not a language trap. `array<T, N>` performs no runtime size computation (N is a constant-expression sized at monomorphization); a monomorphized array whose size exceeds the frame limit is a compile-time rejection [DIAG-1], so `array_new` is `pure`.

## 8. Functions, generics, contracts

[FN-1] Signatures state everything callers need: parameter modes/types, return mode/type, effect row, region parameters. Bodies are checked against signatures; callers rely only on signatures. Function-signature visibility is the [TYPE-6] table.

[FN-2] Generics are monomorphization-only; instantiation arguments are always explicit; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.

[FN-3] Contracts: a `contract` declares fn signatures and laws; `conform T : C { member = fn; }` declares conformance, checked per member; at most one conformance per (type, contract). The prelude marker contracts `Int` and `Float` [PRE-1] carry built-in closed conformer sets (`Int`: i8 i16 i32 i64 u8 u16 u32 u64; `Float`: f32 f64), not user `conform` declarations; a gparam bound `T: Int` (resp. `Float`) makes the integer (resp. float) operation-table rows [OP-1] and the identity literals `0_T`/`1_T` [FORM-5] available for `T`, monomorphized to the concrete type's ops. The exact predicate-vs-method encoding of built-in numeric contracts is a recorded refinement coupled to the generics layer.

[FN-4] A law of a source conformance is admitted only through the mandatory closed discharge below. A successful discharge is source-acceptance evidence, not optimizer authority. For a domain D, totality means every application to values in D terminates without trapping and returns a value in D. A law-table row also defines its result-equivalence relation `≡D`. The checked equations are `f(f(x, y), z) ≡D f(x, f(y, z))` for `associative`, `f(x, y) ≡D f(y, x)` for `commutative`, and both `f(e, x) ≡D x` and `f(x, e) ≡D x` for `identity`, universally quantified over D. `pure` alone proves neither totality nor an equation [EFF-3].

For FN-4 only, the following local binding relation is complete. Source-law discharge in v0.10 requires a nongeneric enclosing contract and a concrete `conform D : C` with no contract type arguments, where D is one concrete integer type and the conformance's C reference resolves to exactly the one enclosing `contract_decl` that owns the law. The law's `f` role equals the name of exactly one `fn_sig` in C. The conformance contains exactly one `fn_bind` whose left IDENT is that name, and its right IDENT resolves under [FN-1] to exactly one top-level `fn_decl`. D, both `fn_sig` parameter types and its return type, and both `fn_decl` parameter types and its return type are the same concrete integer type. Both signatures have exactly two `own D` parameters in corresponding ordinal positions, `own D` return, no region parameters, and effect `pure`; their parameter identifiers need not be equal. The bound function is nongeneric and has no `requires` block. A missing, ambiguous, or different contract resolution, a missing or duplicate referenced member or binding, an unresolved right-hand name, a generic contract or contract argument, a conformance-subject mismatch, or any signature mismatch is a hard error citing FN-4 and publishes no accepted law. This relation decides only the one referenced law obligation; it neither defines nor authorizes whole-conformance acceptance, completeness or validity of uncovered members and bindings, law-free conformance acceptance, generic contract substitution, or behavior-parameterized calls.

After an optional leading `doc`, the bound function's body must contain exactly one statement, `return iadd.sat<D>(p0, p1);`. In this metanotation, `p0` and `p1` mean the exact identifiers declared by the bound function's first and second parameters; they are not required source spellings. Each is used as one bare place, once and in declaration order, and the explicit type argument is D. No alias, field, dereference, `move`, reordered argument, extra statement, second operation, user call, or semantically equivalent body matches this discharge shape.

The v0.10 law table is closed:

| resolved table operation | complete domain D and `≡D` | total | associative | commutative | identity |
|---|---|---|---|---|---|
| `iadd.sat<T>` for T in `u8 u16 u32 u64` | every integer in `[0, 2^K-1]`; same integer value | yes | holds | holds | zero of T |
| `iadd.sat<T>` for T in `i8 i16 i32 i64` | every integer in `[-2^(K-1), 2^(K-1)-1]`; same integer value | yes | refuted | holds | zero of T |

Here K is T's bit width. An `identity` argument matches the row exactly when it is a same-typed [FORM-5] literal denoting T's zero or an IDENT naming an earlier same-typed [CONST-2] value whose substitution result is that zero. Unsigned saturating addition is `min(2^K-1, x+y)`, which makes the three `holds` cells valid over the complete unsigned domain. The signed associativity cell is refuted for every listed width by taking x as `MAX`, y as `1`, and z as `-1`: the left association is `MAX-1` and the right association is `MAX`. Every operation, domain, law, or identity absent from a `holds` cell is unavailable for source discharge in v0.10; a `refuted` or unavailable requested law, or a member function outside the exact discharge shape, is a hard error citing FN-4. This deliberately bounded calculus is part of language acceptance and is identical in every conforming compiler; a compiler's optional prover strength cannot accept more source. Each successfully discharged `(contract-law node, concrete-conformance node)` pair contributes exactly one canonical base derivation record, containing references to its conformance, contract law, bound function, operation row, concrete domain, law, and optional identity [DIAG-2]; no pair is omitted, shared, or deduplicated. Mandatory same-kernel artifact replay recomputes that acceptance judgment, but the record grants no lowering consequence.

A law may affect optimization only through a separately approved optional fact family whose independent verifier binds the accepted base artifact, target, backend, exact proposition, and authorized consequence. For a source law, that verifier independently rederives the complete contract/member/body/table/identity relation above from accepted base-artifact contents; it does not trust the producer's derivation record or same-kernel replay verdict. For a gated law, it validates the exact ledger-entry identity and scope in addition to the proposition. Absence, rejection, or resource failure in that optional path leaves source acceptance, semantic identity, explicit checks, and canonical empty-overlay lowering unchanged. A pre-approved opaque gated-family signature may separately carry a candidate law proposition through its soundness-obligation ledger [LEDGER-1], but that proposition is not a source `conform` discharge and reaches no optimizer without the same independently verified optional-fact boundary. General source proof artifacts, additional operation rows, and other complete-domain proof calculi are DEFERRED specification additions. Sampling, bounded testing, runtime enumeration, and the non-normative law-test harness may prioritize a future gate review but never license optimizer use.

The grammar accepts an IDENT law name and zero or more `law_arg` nodes so that syntax formation does not encode a semantic name, arity, or argument-role table. The checker then requires the name, arity, and argument roles to equal exactly one row of this closed declaration table: `associative(f)`, `commutative(f)`, or `identity(f, e)`. An `f` role is an IDENT resolving to one `fn_sig` declared in the enclosing contract; that signature has effect row `pure`, has exactly two parameters, and gives both parameters and its return the same mode and type. An `e` role is a literal of that type or an IDENT resolving under the ordinary declaration-before-use rule to a named const of that type, and it must be usable at the operation's mode under the ordinary typing and ownership rules. An unknown law name, wrong arity, wrong argument kind, unresolved role, a non-pure signature, a signature that lacks this exact same-mode/same-type binary shape, or role type/mode mismatch is a hard error citing FN-4. A well-formed law in a contract with no concrete conformance is a legal stated obligation and emits no accepted-law evidence. For each concrete conformance, the resolved member must obtain the complete mandatory discharge above before its law obligation is accepted or its accepted-law record is emitted; whole-conformance acceptance remains outside this local relation.

[FN-5] No function values, no dynamic dispatch in the kernel. Behavior parameterization is generics over contract-conforming types (env-struct pattern); closed-set dispatch is `match`. Env-struct calls are guaranteed direct calls after monomorphization (never fn-pointer indirection). Typed operation tables and the mandated env-struct exactness diagnostics are DEFERRED constructs with recorded deltas.

[FN-6] Recursion is permitted. Polymorphic recursion is rejected by a syntactic rule: in any call cycle among generic functions, every call instantiates the callee at exactly the caller's own type parameters. This criterion is DELIBERATELY stronger than finiteness requires (it rejects some finite permutation cycles): predictable, locally explainable rejection per OWN-8's reject-and-restructure posture; the diagnostic must name the cycle and the restructuring. Rejection-rate measurement is a registered experiment.

[FN-7] Exactly one `fn main() -> unit` with effect row at most `allocates(heap), traps` must exist. There is no global state and no `'static` region in v0: ambient mutable globals would (a) erode the noalias fact base every function otherwise gets from parameter-only reachability (P0; carding backlog: GlobalsAA-class evidence), (b) create hidden inter-function channels invisible in signatures (W3, FN-1 signatures-as-trust-unit), and (c) pre-seed shared state for the future concurrency layer (T1). Immutable `const` items [CONST-2] are permitted and are not global mutable state: being read-only they never erode the noalias fact base (reads of frozen rodata add no aliasing hazard), create no hidden inter-function channel (the value is source-determined in the closed unit), and are Shareable-by-construction [CAP-1]; no `'static` region is introduced (borrows of const-rooted places obey the OWN-10 const clause), and there remains no writer-mutable global and no `static mut` analog.

[FN-8] A concrete `fn_decl` may carry one `requires` block after its effect row; the fixed grammar terminal `requires` is ineligible for IDENT under [FORM-3]. The grammar deliberately admits every `doc` or `stmt` as the selected child of a direct `requires_entry`; syntax formation does not encode the block's semantic subset. Before recursively checking any entry, an early FN-8 structural pass requires those selected children to form zero or more `let_stmt` nodes whose selected right-hand side is `ordinary_let_rhs`, followed by exactly one final `check_stmt`, and nothing else. The pass examines direct entries from left to right: every entry before the final position must select an admitted ordinary let, and the final entry must select a check. The first entry that violates that shape is reported; an empty block or an all-let sequence instead reports the `requires_block` node for its missing final check. Thus a nonfinal or repeated check, a `doc`, a `try_let_rhs`, a `value_match`, or any other direct statement shape is a hard error citing FN-8 before any child semantic error can win. The block is a checked callee-entry prologue, not an assumption and not a caller proof obligation: every invocation executes it once after parameter binding and before the function body, including an invocation entering through a gated foreign boundary; a false final condition traps under [OP-5]/[EFF-4], and a true condition contributes its checked fact only to the dominated function body. Ordinary call acceptance never depends on proving the condition. Its scope initially contains only the function parameters; each let introduces a fresh clause-local own copy value visible to later clause statements, and clause locals are not visible in the body. Every computation in the block must be an ANF [GRAM-9] call to a non-trapping, total operation-table row with effect `pure`; the final check condition is either a Bool clause atom or one such call returning Bool. User-function calls, construction, `move`, borrowing, `index`, mutation, control flow, allocation, and any trapping operation are rejected citing FN-8; a place is legal only as a non-consuming operand of an admitted table operation (for example `len<u8>(deref(out))`). Normal typing, ownership, and no-shadowing rules still apply after the structural pass succeeds. The final statement has exactly [OP-5] semantics; a deterministic proof from its passed fact may eliminate only downstream implicit checks such as [OP-4] bounds checks. `requires` is absent from `fn_sig` in this concrete-only first slice and cannot discharge a law under [FN-4]; contract/refinement support is DEFERRED with a recorded delta.

## 9. Effects (gated on exemplar carding before ratification)

[EFF-1] Row grammar: `effects := "pure" | effect ("," effect)*` with `effect := "reads" "(" REGIONID+ ")" | "writes" "(" REGIONID+ ")" | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "traps"`, in exactly this canonical order (reads, writes, allocates, traps). `pure` is the unique spelling of the empty row. Frame residency (STOR-1) is not an allocation by definition.

[EFF-2] Exhibits is syntactic over the complete concrete function declaration: its body and optional `requires` block exhibit `traps` iff either contains any `.trap` op, `check`, a bounds-checked `index`, or a call to any operation or function whose effect row includes `traps` (even if later proven away); they exhibit reads/writes/allocates per the operation table and borrow modes they use. Rows are checked both ways against the syntactic definition: undeclared-but-exhibited and declared-but-unexhibited are both errors. Because [FN-8] requires one explicit `check`, every function with `requires` exhibits `traps`; proof-driven downstream check elision never tightens that row.

[EFF-3] `pure` licenses deduplication and reordering of calls with equal arguments. Elimination of an unused pure call additionally requires a termination proof; v0 provides no termination checker, so unused pure calls are not eliminated. `pure` excludes traps and all reads/writes/allocates; it does not promise termination.

[EFF-4] Trap is abort: no unwinding, no cleanup semantics; the trap report (§12) is the only post-violation artifact.

## 10. Errors

[ERR-1] Recoverable errors are values: prelude `Result<T, E>` and `Option<T>` (§15), dispatched by `match`. No exceptions, no unwinding, no panic values.

[ERR-2] Every `match` is exhaustive over declared variants; there are no wildcard arms. Variant addition surfaces site-enumerated edit lists (toolchain contract).

[ERR-3] Propagation: `let x: own T = try e;` requires `e : own Result<T, E>` and the enclosing function's return type `own Result<U, E>` (same E — no conversions, TYPE-4). On `Ok(v)` it binds v; on `Err(err)` the function returns `Err(err)`, and the elaborated artifact attaches an auto-derived context record (function, node path) to the propagation edge — zero hand-written tokens per site, verifier-checked. Derivation: R4 (keeps recoverable errors shift-left; manual re-match boilerplate invites silent context loss), W1 (one mechanical pattern), W3 (propagation cannot drop the error).

[ERR-4] Classification: expected environment/input failures are values (`Result`); contract violations trap [SCOPE-4]. An operation's classification is fixed by the operation table, never by call site.

## 11. Programs, closed world

[PROG-1] One closed compilation unit formed by [PROG-2]; every language name is defined within it or by the prelude (§15). There is no include, import, module, separate compilation, incremental semantic cache, internal ABI, dynamic loading, reflection, or source-path lookup in the language. A logical source path contributes identity only and never a namespace or lookup key. The only external boundary is the gated FFI wall (§14).

[PROG-2] One compilation unit is one ordered nonempty sequence of logical source records. Each record contains one logical path and one exact source-byte sequence. A logical path is an ASCII relative path made from one or more nonempty components separated by exactly one `/` byte, with no leading, trailing, or repeated `/`; each component contains only ASCII letters, ASCII digits, `.`, `_`, or `-`, and no component is `.` or `..`. Path spelling is preserved exactly and compared case-sensitively. An empty record sequence, an invalid logical path, or two records with the same logical path is an input-envelope failure, not a source-language rejection. Record order is exactly the order in the bound invocation; no path sort, host enumeration order, or other reordering is applied. Within that bound unit, a source record is identified by its zero-based ordinal, exact logical path, and exact source bytes.

Every record is parsed as an independent [GRAM-2] `item*` sequence and audited as an independent [FORM-2] source. A zero-byte source is a valid input record whose empty `item*` derivation fails [FORM-2]; the sole canonical zero-item source is exactly one LF byte. No token, trivia item, grammar production below the compilation-unit `program` root, or source span crosses a record boundary. The toolchain inserts no token, whitespace, delimiter, declaration, or separator between records.

The `program` root defined by [GRAM-2] owns the flattened sequence of all item nodes, ordered first by source ordinal and then by source-local item order. Its location extent is `BundleRootExtent`, the ordered sequence `(source_ordinal, 0, source_byte_length)` for every record, including records with no item nodes. It is not a fabricated source-local span. Every descendant is source-local, and source records are not grammar-tree nodes. Canonical formatting is checked separately for every record. A record boundary and an empty record remain part of the bound source identity even when they contribute no item; repartitioning or reordering the same item bytes therefore changes that identity.

Top-level declaration order is source ordinal followed by source-local item order. Name visibility is exactly the [TYPE-6] table. Global uniqueness, the prelude, `main`, conformances, call graphs, strongly connected components, concrete instances, and reports range over the entire closed compilation unit. Logical paths and record boundaries introduce no namespace, scope, import, or lookup key.

## 12. Diagnostics and artifacts (toolchain floor)

[DIAG-1] Every source-language rejection cites exactly one numbered language rule and exactly one location from this closed sum:

1. `SourceBytes(SourceCoordinate)` when no offending canonical-tree node exists or the defect belongs only to a source boundary;
2. `SourceNode(NodePath, SourceCoordinate)` when one source-backed canonical-tree node is the offending node; or
3. `BundleRoot(NodePath, BundleRootExtent)` for a whole-unit defect with no offending source declaration. This form requires the empty root `NodePath` and carries no source-local byte interval.

`SourceCoordinate` is `(source_ordinal, byte_start, byte_end)` in the bound [PROG-2] unit. Its byte interval is checked, half-open, and contained in that exact source. End of source is the zero-width interval whose two offsets equal the source byte length. `NodePath` is the sequence of zero-based child ordinals from the finalized compilation-unit root; the root path is the empty sequence. Every source-backed node has one checked source-local extent. In `SourceNode`, the rule-selected coordinate lies within that extent (`node_start <= byte_start <= byte_end <= node_end`) but need not equal the complete extent; the path identifies the existing offending or owning node while the coordinate identifies its exact offending subinterval or boundary. `BundleRootExtent` is the exact ordered byte-extent sequence defined by [PROG-2], not a cross-source byte span. A diagnostic never fabricates a node or node path. A nested-place rejection additionally renders the offending access-path segment.

The frontend selects defects stage by stage. Each stage scans every source in source-ordinal order and byte order, stops at its first defect, and the next stage begins only if the preceding stage succeeds for every source. The stage order is: raw lexical formation; terminal membership; grammar derivation; then canonical [FORM-2] rendering. Within one grammar decision, production definitions rank by their first appearance in this specification, and alternatives rank left to right as written. Numbered rules rank by their first appearance in this specification.

Raw lexical scanning is quote-aware and reports the first defect at its cursor. If the actual byte sequence beginning at the cursor does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, the first byte always cites [FORM-2] and spans that one byte, including when the cursor is inside a STRING candidate. Outside a STRING candidate, a byte in `0x00..0x1f` other than LF, or byte `0x7f`, cites [FORM-2] and spans that byte. An exact `//` or `/*` prefix outside a STRING candidate cites [FORM-4] and spans those two bytes. A `'` or `@` not followed by `[a-z]` cites [FORM-3] and spans only the sigil. Any other ASCII byte that cannot begin a specified token cites [FORM-1] and spans that byte. Any valid non-ASCII scalar outside a STRING candidate cites [FORM-1] and spans its complete UTF-8 encoding.

After an opening `"`, `//` and `/*` are ordinary raw STRING bytes and never comment prefixes. A final backslash cites [FORM-5] and spans only that backslash. A backslash followed by an ASCII byte other than `\`, `"`, or `n` cites [FORM-5] and spans both bytes. If the actual byte sequence beginning at a backslash's follower does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, that follower instead cites [FORM-2] and spans only its first byte; if the follower begins a valid non-ASCII scalar, [FORM-5] spans the backslash and that scalar's complete UTF-8 encoding. A raw ASCII byte outside the permitted STRING interior set cites [FORM-5] and spans that byte. At any other STRING cursor, if the actual byte sequence beginning there does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, [FORM-2] spans its first byte; a valid non-ASCII scalar instead cites [FORM-5] and spans its complete UTF-8 encoding. If no unescaped closing quote occurs and no earlier defect applies, the unterminated STRING cites [FORM-5] and spans from its opening quote through end of source. Terminal membership uses the complete context-free predicate set required by [GRAM-1]; a token with no matching predicate cites [FORM-3] or [FORM-5], whichever rule owns the rejected spelling. Every lexical, terminal-membership, or grammar rejection uses `SourceBytes`; its coordinate is the exact interval above, the exact offending token interval, or the zero-width end-of-source interval defined above.

Every grammar production and external terminal predicate is owned by the numbered rule containing its unique definition. A source-EBNF decision is a `|`, `?`, `*`, or the continuation decision of `+`. Its stable identity is the zero-based ordinal of its production by first definition in this specification followed by the zero-based EBNF child-index path from that production's root. Its arms retain source order; a consuming arm precedes an exit arm. The strong-LL(2) analysis required by [GRAM-1] supplies every arm's `SELECT_2` rows. Every predicate in a row retains its source-EBNF provenance and whether it came from inside that arm or from the arm's caller continuation. Lookahead is padded to two positions with `SOURCE_END`.

Recognition selects an arm only when that arm has a full two-position row match. Two matching arms are a [GRAM-1] specification defect, not a precedence rule. Whenever no row matches, the diagnostic machine computes each arm's score: the greatest proper-prefix length, zero or one, by which any of that arm's rows accepts the actual two-position lookahead. Let `m` be the greatest score at that frontier. The failure boundary is the actual lookahead token at position `m`, or the zero-width end-of-source coordinate when that position is `SOURCE_END`. The maximal-prefix rows are every row with score `m`. The expected-terminal set is the distinct predicates at position `m` in those rows, ordered by their first terminal occurrence in the approved grammar; written terminals precede `SOURCE_END`. A direct terminal mismatch is the same calculation with one row and has a singleton expected set.

At every no-row frontier, the following closed attribution rows are tested in order before diagnostic traversal descends. The first matching row stops traversal. A row retains the frontier expected-terminal set and coordinate unless that row names a replacement.

1. If the boundary token is one member of four consecutive actual tokens `IDENT "." IDENT ("("|"<")`, that dotted call-or-targs spelling cites [FORM-3]. Its coordinate is the complete interval from the first IDENT through the second IDENT. An allowed suffix would already be one maximal OPNAME token, while a field place cannot be called or given targs. This bounded diagnostic window may include already recognized tokens, performs no operation-table or name lookup, consumes nothing, and does not enlarge recognition's two-token lookahead.
2. If source-EBNF provenance reaches or would next enter an `atom` occurrence in `atom_list`, `fieldinit`, or the `index` offset, and the two actual tokens at the start of that occurrence are `(IDENT, "(")`, `(IDENT, "<")`, `(OPNAME, "(")`, `(OPNAME, "<")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the rejection cites [GRAM-9]. These are exactly the `call` and `construct` starts forbidden in an atom-only position; no name lookup participates. Its coordinate is the complete interval from the first through the second token of that forbidden call or construct start.
3. If the boundary token has the raw shape admitted by an expected external predicate before that predicate's explicit spelling restrictions, and fails only those restrictions, the rejection cites that predicate's owner. This includes an exact fixed lowercase grammar word in an IDENT slot and a numeric-form token missing FORM-5 membership. For the rest of this row, a boundary-name candidate is one of `IDENT`, `TYPEID`, `REGIONID`, `LABEL`, or `OPNAME` when the boundary token satisfies a different predicate in that five-member set. A transparent mandatory-name path begins at a position-m predicate occurrence in one of the current frontier's maximal-prefix `SELECT_2` rows, using that occurrence's source-EBNF provenance; it never restarts at the failed decision's head. The path ends at a boundary-name candidate which is its first nonnullable unconsumed terminal. It may traverse a group; a sequence whose preceding children are completely matched; or a production reference whose expansion before that terminal contains no source `|` decision. At a `?`, `*`, or `+` continuation it examines both the consuming direction and the exit/caller-continuation direction recursively. The nullable decision is transparent only when no direction's first nonnullable unconsumed predicate accepts the boundary token. Every direction that recursively reaches a boundary-name candidate contributes a path; a direction that instead reaches a different nonmatching predicate contributes none. A path stops at any source `|` or at any nonnullable terminal before its candidate. A name-slot mismatch exists only when at least one transparent path exists and every transparent path ends in the same name predicate. It cites [FORM-3]. Thus traversal reaches a direct name, a name inside a consuming list arm, or a name after one or more skipped nullable prefixes such as `doc?`, but cannot tunnel through structural choices such as `item`, `stmt`, `expr`, `atom`, `callee`, `pbase`, `targ`, `law_arg`, `requires_entry`, or `atom_list | fieldinit_list`. If several external predicates qualify under the first sentence, their owners rank by first rule occurrence in this specification.
4. At the `program` `item*` or `item` entry, any `stmt*` or `stmt` entry, or the `requires_entry*` or `requires_entry` entry, an IDENT-headed lookahead accepted by no complete construct row cites [FORM-1] as an unknown construct. Its coordinate is the exact interval of that first IDENT token. A lookahead that selects a defined construct is not covered by this clause.
5. At `program`'s `item*`, after any complete item prefix, if the first actual token predicate matches no consuming `item` row, the token is an unexpected leftover, the expected-terminal set is replaced by only `SOURCE_END`, and the rejection cites the owner of `program`.

If no attribution row applies and exactly one arm has a score strictly greater than every other arm, diagnostic traversal descends into that arm only when every next expected predicate in that arm's maximal-prefix rows came from inside the arm rather than from its caller continuation. Otherwise the current frontier is the stopping point. A tie is never guessed. Traversal through the selected arm follows the same source EBNF and repeats this procedure. It cannot cross from a completed arm into its continuation, make a failed row valid, insert, delete, recover, or skip a token, or create a derivation or node. It is used only after recognition has failed; reaching a successful end instead of a stopping point is a compiler-invariant failure.

At a stopping decision the total fallback cites the owner of the production containing that source-EBNF decision; a direct terminal mismatch cites the owner of its containing production. A recursive-descent or table-driven implementation must report the result of this same source-EBNF diagnostic machine.

A source-local trivia gap is the complete interval between two adjacent terminal leaves after excluding those terminal bytes, between source start and the first terminal, or between the last terminal and source end. It contains every intervening trivia item and may be zero-width; for a source with no terminal leaves, the whole source is its single gap. The forest renderer defines the required bytes for each corresponding boundary. A [FORM-2] mismatch is selected by the first byte offset at which the source and its complete forest rendering differ, treating the end of either byte sequence as a boundary. Because terminal bytes have already passed lexical formation, terminal membership, and grammar derivation, that offset selects exactly one such actual-or-required gap. Its coordinate is the complete actual gap interval, or the zero-width terminal boundary when required trivia is missing. For a gap between two adjacent terminal leaves in the same top-level item, the location is `SourceNode` for their deepest common production-node ancestor in the finalized compilation-unit tree. A source-leading, source-final, inter-item, or zero-item-source gap uses `SourceBytes`. No renderer-authored owner, parser stack position, or implementation emission order participates in this choice.

An input-envelope failure, resource failure, compiler-invariant failure, untrusted-artifact failure, backend failure, or external-tool failure is not a source-language rejection, cites no language rule, and carries no expected-terminal set.

After canonical FORM-2 succeeds for every source, semantic diagnostic selection first runs the early FN-8 structural-admission pass over every `requires_block`. Within a block, FN-8 selects its specified first invalid direct `requires_entry` or the block node for a missing final check. An invalid direct entry uses `SourceNode` at that `requires_entry` production and a `SourceCoordinate` equal to that production's complete checked half-open source extent. An empty or all-let block missing its final check uses `SourceNode` at the `requires_block` production and a `SourceCoordinate` equal to that block production's complete checked half-open source extent. These are existing owner-production extents under DIAG-1; neither case fabricates a child node, a zero-width boundary, or a terminal-only coordinate. Across blocks, the minimum tuple `(source_ordinal, byte_start, byte_end, NodePath)` of that selected location wins. Numeric fields compare ascending and NodePath compares as defined below. No declaration or use role inside an inadmissible block is classified or counted. Only complete unit-wide FN-8 admission permits role classification and its exact resource-count derivation, only complete FN-8 admission permits declaration inventory, and only complete inventory permits lexical resolution. Poison declarations and partial resolution are forbidden. An FN-8 rejection outranks every inventory or resolution rejection; an inventory rejection outranks every resolution rejection even when the later-stage event has an earlier source coordinate.

A semantic role is owned by the lowest production node whose selected right-hand side directly contains the terminal that carries the role; a role reached only through a referenced child production is owned by that child. A referenced child production means a child production node, not an external terminal predicate such as `literal`. A semantic role may occupy a complete name terminal, a complete literal terminal, or the exact TYPEID suffix of a FORM-5 generic numeric literal `0_T` or `1_T`. The suffix role's spelling excludes `_`, and its coordinate is exactly the suffix byte interval. One token may carry more than one role: for example, a law argument `0_T` has one deferred law-argument role on the complete literal and one lexical generic-type use on `T`. A struct TYPEID remains one declaration event producing two domain entries, not two events.

Within one owner node, distinct direct grammar-role carriers are ordered left to right by their complete carrier coordinates; distinct carriers with identical complete coordinates use the closed class order declaration, lexical-use, deferred-use. The zero-based carrier index is `role_ordinal`. `subtoken_ordinal` is zero for a role covering its complete carrier; embedded semantic name roles are numbered from one in byte order. The only multi-role carrier is X09/U18, where the class tie does not reorder the embedded role: a law-argument `0_T` gives its complete deferred argument `(role_ordinal, 0)` and its embedded generic-type use `(role_ordinal, 1)`. Every role has exactly one owner, class, role ordinal, and subtoken ordinal. Every declaration, lexical-use, and deferred-use event has canonical key `(source_ordinal, byte_start, byte_end, NodePath, role_ordinal, subtoken_ordinal)`. Numeric fields compare ascending. NodePath compares lexicographically by production-child ordinal, with a proper prefix first. Role and subtoken ordinals are consulted only after the complete path is equal. For a complete IDENT, TYPEID, OPNAME, REGIONID, LABEL, or literal role, the coordinate is the complete token interval, including a sigil; only the generic-numeric suffix uses a subtoken coordinate. The event's `SourceNode` names its owner production. Traversal order, allocation identity, map order, logical path, and inferred type never participate.

Declaration inventory creates candidates under this closed rank:

1. a FORM-3 reserved-name violation defined by OP-1's derived set;
2. an OWN-3 repeated REGIONID declaration within one function declaration or contract-member signature, parameters included;
3. a GRAM-10 match-binder freshness violation;
4. a TYPE-6 collision with PRE-1;
5. a TYPE-6 compilation-root duplicate or same-lexical-scope redeclaration; and
6. a TYPE-6 nested declaration shadowing a live declaration.

The stage selects the minimum declaration-event key and then the first applicable rank at that event. A FORM-3 reservation payload is `(spelling, declaration_role, reserved_class, inventory_ordinal)`. Its `spelling` is the complete declaration spelling except that a REGIONID uses its unsigiled IDENT-shaped interior while the rejection coordinate retains the complete sigiled token. Its closed declaration roles are function, named-const, parameter, let, match-binder, field, variant-field, region-parameter, and local-region. `reserved_class` is dotless-operation or mode-word. A dotless-operation ordinal is the zero-based first occurrence among distinct operation-family spellings, scanning OP-1 rows top to bottom and each `op` cell left to right and skipping every later occurrence of the same spelling; both `cvt` rows therefore name one family and one ordinal. A mode-word ordinal is the zero-based FORM-3 alternative order `wrap`, `trap`, `checked`, `sat`, `strict`. Those two reserved sets are disjoint in this version. An OWN-3 repeated-region payload is `(spelling, conflicting_region_origin)` and points to the later region declaration; OWN-3 precedes GRAM-10 in the rank even though no grammar carrier can be both a region declaration and a match binder. For the GRAM-10 violation defined by TYPE-6, the payload is `(binder_spelling, paired_field_spelling, optional_earlier_binder_origin, ordered_arm_entry_live_lexical_ident_origins)`. Earlier binders and arm-entry origins are ordered by declaration-event key. That binder does not also create a TYPE-6 duplicate or shadow candidate.

A TYPE-6 collision payload is `(spelling, ordered_nonempty_conflicts)`. Conflict domains use the fixed order lexical-IDENT, nominal-type, constructor, contract, REGIONID, LABEL. Each conflict contains its domain, declaration class, and `conflicting_origin`; conflicts within one domain use PRE-1 declaration ordinal first and then source declaration-event key. A source origin is `(NodePath, SourceCoordinate, role_ordinal, subtoken_ordinal)`; a PRE-1 origin is `(PRE-1, declaration_ordinal)`, where `declaration_ordinal` is the zero-based twenty-four-record preorder fixed by TYPE-6. A struct event may report both nominal-type and constructor conflicts in that order. Rank 4 reports only PRE-1 conflicts when the same event also conflicts with source. A PRE-1 collision points to the source declaration. Rank 5 points to the later source declaration event. Rank 6 points to the nested declaration, including one shadowing a source-later but whole-unit-visible function. Every inventory rejection uses `SourceNode` at the declaration role and has no expected-terminal set.

If inventory succeeds, every lexical use admitted by TYPE-6 or OP-1 creates one lexical-use event. The generic-numeric suffix admits a live generic TYPEID parameter; FN-3 and FORM-5, not lexical resolution, later require its numeric bound. Lexical resolution fixes only the declaration or operation-family target.

The closed declaration-class order is function, named-const, const-generic, value, generic-type, nominal-type, struct-constructor, enum-variant, contract, region, label, operation-family. TYPE-6 and OP-1 fix each lexical role's ordered admissible subset. A use's exact-spelling candidate universe contains all compilation-root entries in its grammar-selected domain and, for non-root declarations, only entries belonging to its declaration-owner chain. All sibling or expired lexical scopes within the same `fn_decl` owner participate so that an out-of-scope same-function declaration can be distinguished from absence. A contract-member signature admits declarations of that signature and its enclosing contract ancestry but not declarations owned only by a sibling member signature. A struct, enum, contract, or function generic belongs only to that declaration and its descendants. No local, generic, parameter, region, or label owned solely by an unrelated top-level declaration or function participates. PRE-1 owner-local type parameters and fields never participate in source lookup. LABEL uses instead follow the separate current-function rule below.

For one lexical-use event the closed lookup rank is:

1. the candidate universe has at least one declaration in an admissible class but its admissible visible subset is empty; cite the role-attribution table below and carry every invisible admissible origin in declaration-event order;
2. for LABEL only, the current function has at least one exact-spelling label but none declares a loop lexically enclosing the `break`; cite TYPE-6 and carry every such current-function label origin in declaration-event order; and
3. the visible admissible subset is empty and neither rank 1 nor rank 2 applies; cite the role-attribution table below.

| lexical-use role | rule cited by rank 1 or rank 3 |
|---|---|
| `type` TYPEID | TYPE-5 |
| contract bound or `conform_decl` contract TYPEID | FN-3 |
| `construct` constructor TYPEID or enum-variant-only `arm` TYPEID | TYPE-6 |
| REGIONID use | OWN-3 |
| LABEL use | TYPE-6 |
| `const` IDENT | CONST-1 |
| `cvalue` IDENT | CONST-2 |
| `pbase` IDENT | TYPE-5 |
| IDENT or OPNAME `callee` | OP-1 |
| `fn_bind` right IDENT | FN-4 |
| FORM-5 generic-numeric TYPEID suffix | FORM-5 |

A successful non-LABEL lookup has exactly one visible admissible target; a successful LABEL lookup has exactly one enclosing target. A rank-1 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_nonempty_invisible_origins)`. A rank-2 payload is `(spelling, lexical_use_role, ordered_nonempty_label_origins)`. A rank-3 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_available_classes)`, where available classes are visible exact-spelling entries in that use's candidate universe, listed once in the closed class order and possibly empty. Complete IDENT, TYPEID, OPNAME, REGIONID, and LABEL use spellings include any sigil; only the generic-numeric suffix spelling is bare `T`. This is declaration-kind resolution, not type checking. Across use events the minimum event key wins. Every resolution rejection uses `SourceNode` at the use role and has no expected-terminal set.

The dependent-declaration carriers are exactly the `field` and `vfield` declarations and the member declaration of `fn_sig`. Each is a declaration-class carrier that produces one dependent-declaration record and one declaration event for later typed owner/member checking, but none enters a resolver lookup inventory. The two field carriers participate in FORM-3's reservation inventory; the contract-member carrier does not. The deferred-use carriers are exactly the `law` name and each complete law argument, the left IDENT of `fn_bind`, the first IDENT of `fieldbind`, each `fieldinit` IDENT, and each `psuffix` IDENT. Each produces one deferred-use record for later typed owner/member checking. The lexical generic suffix inside a deferred literal law argument additionally receives its ordinary lexical-use record; this X09/U18 pair is the only same-token overlap and produces two distinct role records. In an `arm`, its leading TYPEID first resolves globally to an enum variant; later typed checking compares that variant's owning enum with the scrutinee enum, and a foreign-variant relation cites TYPE-6. The resolver does not otherwise accept or reject a dependent role's owner/member relation.

A missing whole-unit requirement is not fabricated as an inventory or lookup event. Missing `main` remains an FN-7 rejection at `BundleRoot`. Duplicate `main` names are the later-source TYPE-6 duplicate; one unique but wrong-signature `main` is a later FN-7 rejection at its source declaration. Missing or duplicate contract members, field labels, conform bindings, and law roles remain typed-dependent rejections. Selection order for semantic and target stages after lexical resolution must be separately approved with those stages.

A mechanical fix or restructuring is included exactly where the owning rule requires one. Every published diagnostic is deterministic and byte-stable.

[DIAG-2] Accepted programs elaborate to a canonical artifact where every derived operation (drops, arena releases, instantiations, retained checks) is explicit; the artifact embeds proof objects and check instrumentation so acceptance is decidable from the artifact alone.

[DIAG-3] Normative report family accompanying the artifact, with field schemas:

| report | fields (all required) |
|---|---|
| trap | rule_id; message; node_path; function; stack_attribution (frame list: function, node_path); artifact_hash |
| check | function; per check: node_path, fact_class (bounds/overflow/alias/user), status (retained/eliminated), proof_ref (for eliminated: checker-derivation id) |
| lifetime | function; per binding: name, mode, region, drop node_path (artifact-explicit) |
| check-density | per function: checks_retained, checks_eliminated, elimination_ratio |

Reports are machine-readable, deterministic, and byte-stable for identical artifacts [DIAG-1]; the writer-facing schemas above are counted spec mass (no-shadow-spec).

## 13. Capabilities (stub; concurrency layer pending)

[CAP-1] Type-level capability predicates of the Send/Sync class exist in the kernel vocabulary: `Shareable` (safe to share across threads) and `Sendable` (safe to transfer). v0 defines no thread construct, so no kernel type is required to declare them; the predicates reserve the vocabulary the concurrency layer will bind. Data-race impossibility is D1 law; general race conditions are out of scope (C004 amended scope).

## 14. Gated family (writer-visible stub)

[GATE-1] Editing any declared contract, signature, law bundle, storage contract, or gated-family member is one privileged, gated toolchain operation with one audit trail, outside steady-state writer capability.

[LEDGER-1] There is exactly one boundary-construct family (unsafe regions, FFI extern frames, trusted primitive imports), sharing one per-fact soundness-obligation ledger; manifest-free members are unrepresentable; members are AI-authored and human-approved through the gate (owner ruling D0a). A kernel writer sees these constructs only as opaque, pre-approved library signatures.

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

## 16. Worked example (normative bytes)

[EX-1] The following complete program is byte-exact canonical form:

```
enum Sign {
  Neg();
  Zero();
  Pos();
}

fn sign_of(x: own i32) -> own Sign pure {
  doc "Conditional value produced by returning from arms (canonical for return position).";
  match ilt<i32>(x, 0_i32) {
    True() => {
      return Neg();
    }
    False() => {
      match ieq<i32>(x, 0_i32) {
        True() => {
          return Zero();
        }
        False() => {
          return Pos();
        }
      }
    }
  }
}

fn main() -> own unit traps {
  doc "let-initializer match with give: a conditional value bound, then reused.";
  let a: own i32 = 40_i32;
  region 'r {
    let p: &'r i32 = &'r a;
    let v: own i32 = match iadd.checked<i32>(deref(p), 2_i32) {
      Ok(value: w) => {
        give w;
      }
      Err(error: e) => {
        return unit;
      }
    }
    check ieq<i32>(v, 42_i32) else trap "arithmetic drift";
  }
  return unit;
}
```

## 17. Spec meta-rules (CI-checked)

[META-1] Spec-CI enforces the regularity invariants defined elsewhere: one spelling per construct [FORM-1] and a 1:1 production-to-core-tree-node mapping [GRAM-1]. Its unique machine-checked content is that no rule ID is defined twice and every cross-reference resolves [META-4, META-6].
[META-2] No context-dependent spellings or rule variants: no rule's meaning depends on surrounding context; defaulting rules do not exist.
[META-3] No rule carries an exception clause; conditional structure is expressed as total positive rules or table data.
[META-4] Every normative fact is stated once; other mentions are rule-ID cross-references.
[META-5] Every change to this artifact declares its spec delta (rules ±, tokens ±, spellings ±, exceptions ±) and its SELECTION GROUND (evidence-selected vs minimality-selected) in the decision gates; DEFERRED markers are tracked delta obligations.
[META-6] Every rule carries an entry in the derivation ledger (spec/derivation-ledger.md) tracing it to CONSTITUTION.md; a rule whose chain is refuted or orphaned (evidence card dies, constitutional premise amended) is automatically flagged for re-grounding; underived rules may not ratify.
