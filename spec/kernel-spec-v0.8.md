# Kernel Specification v0.8

Status: DRAFT v0.8 (2026-07-19; tag-only enum equality). Adds `eeq` and `ene` as pure, total equality and inequality operations over one exact nominal tag-only enum type, including `Bool`; `ieq` and `ine` remain integer-only, and payload-enum equality, enum ordering, cross-enum comparison, and enum/integer conversion remain absent. Evidence and gates: optimizer-language-research/implementation/enum-equality-investigation/PACKET.md. Prior: DRAFT v0.7 (2026-07-18; bounded reborrow). Admits statement-scoped, non-escaping child reborrows: OWN-5 gains a child-creation carve-in and a suspended-holder carve-out, OWN-6 defines the statement-scoped child and its suspension/resumption, OWN-9 tightens the optimizer no-alias note to usable and non-suspended borrows, OWN-12 exempts the suspended ancestor from the effect-row check, new STOR-5 numbers the borrow-free and region-free storage invariant, and PATTERNS P4 becomes bounded reborrow; the deferred reference-result-provenance, uniq-to-shared downgrade, match-binder parents, grandchild chains, and bound children remain out. Evidence and conditions: optimizer-language-research/implementation/reborrow-investigation/PACKET.md. Prior: DRAFT v0.6 (2026-07-08; additive clusters). PROOF-2 first slice (owner-approved 2026-07-11) adds the checked, concrete-function-only `requires` prologue [FN-8]; the construct's existence and callee-entry semantics are evidence-selected, while its block spelling is R3-provisional pending writer/codegen comparison. Sub-step 1 (op-table completion) applied: integer bitwise/shift/rotate/bit-count/mulhi/saturating/min-max/abs, `reinterpret`, and the float math family (fneg/fabs/fmin/fmax/sqrt/floor/ceil/trunc/roundeven/frem/fma) added to OP-1, plus fgt/fge/fne and finf/fnan; OP-7 (op-name convention) and OP-8 (edge semantics + lowerings) added; OP-2/OP-3 restated; FORM-5 gains float-exponent and generic 0_T/1_T literals; Int/Float marker contracts added to the prelude. Sub-step 2 (data stack) applied: `buffer<T>` runtime-length heap value, `array_new`/`buffer_new` (OP-9 traps on size-overflow), `const` items (CONST-2) and a closed const-expr sublanguage (CONST-1 rewritten, re-adding the const-generic gparam that D8 deferred), collections ruled library/out-of-v0. Sub-step 3 (surface) applied: named-in-declared-order construction (positional removed, GRAM-8), three-address/ANF computation (GRAM-9), named match binders (GRAM-10), and the `give` value-match (GIVE-1, GRAM-7 rewritten) that deletes the helper-fn idiom; PRE-1 Option/Result payloads named; EX-1 re-cut. Polish (owner rulings 2026-07-08): named-in-declared-order arguments for user-fn calls (GRAM-11; no reordering, FORM-1), positional operands for table ops; global enum-variant-name uniqueness (TYPE-6); `reinterpret` extended to same-width int<->int resign (OP-1/OP-8); built-in Int/Float numeric conformance (FN-3); R0 confirmed to credit W3/W1 deltas (CONSTITUTION.md). Prior: DRAFT v0.5 (2026-07-07; Tier-0 errata). Fixed 11 internal self-contradictions (D1-D11): EX-1 is now parse/reprint/check-derivable; FORM-3 OPNAME tightened to a closed mode-suffix set with a GRAM-5 callee production (resolving the OPNAME/field-access collision); FORM-2 byte-format completed; FORM-7 (numeric-literal well-formedness), TYPE-7 (explicit deref typing), and OP-6 (cvt exact-or-Result, 29 total pairs, no rounding) added; DivError replaces DivideByZero; integer literals carry an inline sign (iK::MIN now writable); STRING pinned to ASCII-printable; the dead const-param gparam removed; META-1/META-4 dedup. Prior: DRAFT v0.4.1 (2026-07-07). DIAG-3 field schemas added (closes the audit-flagged R4-load-bearing deferral). GRAM-4 scrutinee widened to expr, resolving the GRAM-4/EX-1 contradiction found by M1 construction (bind-then-match rejected: per R3/W1 it taxes the sole conditional idiom with a mechanical temporary at every use); OWN-13 gains the owned-temporary clause. v0.3.1 added ERR-3 propagation (closed the R4-load-bearing deferral). v0.3 was the ledger-fix revision of v0.2 (META-6 derivation discipline; OP-2 ineg.wrap added with corrected rationale scope; FORM-6/FN-6/FN-7 derivation rationales recorded). v0.2 was the lexicon revision of v0.1 (owner rulings: borrow-mode rename mut->uniq; lexicon policy LEX-1). v0.1 was the revision of v0 under the round-1 spec critique (63 findings, 37 missing rules; all blocking and major findings addressed or explicitly deferred with recorded deltas). Section 5 (ownership) is FORMALLY RECONCILED against Featherweight Rust (spec/fr-reconciliation-m0.md: OBL-0..3 all discharged 2026-07-07, incl. verbatim paper pass with page-anchored quotes; the checker fragment is a proven sound subset of FR state space per T-A); the v0.7 bounded-reborrow edge is reconciled as a stricter-sound subset of FR reborrowing (checker state stays singleton-rooted; the lexical statement-scoped suspend is a subset of FR's flow prohibition), evidenced by a 1,000,000-program model-check with zero violations, with the verbatim FR *w-edge paper pass recorded as owed. Owner ratification word pending; no open technical obligations. Section 9 (effects) is gated on region/effect exemplar carding before ratification.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification — see decision-gates.md): loop form (GRAM-5/6), match-only conditionals and no-if (GRAM-6/PRE-1), statement-only match (GRAM-7), prefix arithmetic surface (OP-1/GRAM-6), interior annotation mandate (TYPE-5 — round-2 verdict still needs_evidence), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), deref/index prefix places (GRAM-5), and the `requires { let_stmt* check_stmt }` block spelling (FN-8 — semantics selected, spelling not yet compared).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14). The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is accepted iff it parses under the canonical grammar, satisfies every rule in this document, and every unproven D1-critical checkable fact (bounds; alias-disjointness where a check form exists) carries a runtime check. There is no writer-emittable third state: nothing writer-stated is trusted unchecked. The sole trusted-assertion class is toolchain-gated ledger entries (§14), which the writer cannot author or edit.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code. This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.

[SCOPE-4] Contract violation at runtime traps: the process emits a machine-readable trap report (§12) and aborts. There is no unwinding.

## 2. Canonical form

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting. Non-canonical input is a hard error; the toolchain never auto-formats. Unknown constructs are hard errors (conservative extension).

[FORM-2] Formatting, exhaustively: UTF-8; LF endings; file ends with exactly one LF; indentation exactly two spaces per `{` nesting level (match arms are one level inside `match`); exactly one space between adjacent tokens except: no space after `(` `<` `&` or before `)` `>` `,` `;` `.` `:` `(` `<`; one space after `,` and `:`; no space around `.` in places; no line wrapping (a statement is one line); declarations separated by exactly one blank line; no trailing whitespace.

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*`; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)` (single token; the base is an IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a field-access place `p.field` [GRAM-5]; e.g. `iadd.checked`).

[FORM-4] There are no comments. Documentation is the `doc` field of declarations [GRAM-3]. Provenance lives in toolchain records.

[FORM-5] Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; a leading `-` is legal for signed TYPE, and the signed value must lie in TYPE's range [FORM-7]; e.g. `42_i32`, `-2147483648_i32`); floats `-?[0-9]+\.[0-9]+(e-?[0-9]+)?_TYPE` (a leading `-` is legal for the value; the canonical spelling is the unique shortest decimal digit string that round-trips under round-to-nearest-even, with at least one integer and one fraction digit, lowercase `e`, and no leading zeros; `-0.0` is distinct from `0.0`; e.g. `1.5_f64`, `6.022e23_f64`); `unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, or one of exactly three escapes `\\ \" \n`; no other byte is legal, and each character has exactly one spelling (the escape where one is defined, the raw byte otherwise). STRING appears only in `doc` and `check` messages; non-ASCII diagnostic text is DEFERRED. There are no boolean literals: `Bool` is a prelude enum (§15). Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric contract (`Int` or `Float`, §15), denoting T's additive and multiplicative identity; a concrete type uses `0_i32` and the like, so there is no dual spelling. NaN and the infinities are not literals; they are the nullary ops `fnan` and `finf` [OP-1].

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual. The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling is the R3 one-spelling choice for the type's sole inhabitant.

[FORM-7] Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` is legal for signed T, and `-0` is written `0`. A float literal is legal where its round-to-nearest-even value in T is finite. An out-of-range integer, a leading-zero integer, or a float that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value. The canonical decimal spelling of a float value is gated on the FORM-1 reject-vs-canonicalize decision and DEFERRED.

[LEX-1] Lexicon policy: surface names label checked invariants, stated in this document self-containedly. Names are never borrowed from backend IR vocabulary (e.g. `noalias`), which names lowering consequences, not source invariants; and a name is borrowed from another language's convention only where a divergence census shows the semantics genuinely match. Ruling of record: the exclusive borrow mode is `uniq` (uniqueness-type lineage), not `mut` (Rust divergence: exclusivity is the invariant; mutation is only its permission, and the name breaks under future interior-mutability capabilities). DEFERRED with recorded delta: the two-axis mode vocabulary (exclusivity x write-permission, adding frozen/exclusive-read and capability-gated shared-write).

## 3. Grammar

[GRAM-1] The grammar is deterministic and unambiguous (one parse per input; resolved with two-token lookahead where FIRST sets overlap). Every production maps 1:1 to one core-tree node kind; there is no desugaring.

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
requires_block:= "requires" "{" let_stmt* check_stmt "}"
contract_decl:= "contract" TYPEID generics? "{" doc? fn_sig* law* "}"
fn_sig       := "fn" IDENT region_params? "(" param_list? ")" "->" rtype effects ";"
law          := "law" LAWNAME "(" IDENT ("," IDENT)* ")" ";"
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
const  := "[0-9]+" | IDENT   # bare u64 literal, or in-scope const-param / named-const [CONST-1]
cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]"   # const_decl RHS only; never runtime expr
```

[GRAM-4] Statements:

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | match_stmt | try_stmt
             | give_stmt
try_stmt    := "let" IDENT ":" mode type "=" "try" expr ";"
let_stmt    := "let" IDENT ":" mode type "=" ( expr ";" | match_block )
set_stmt    := "set" place "=" expr ";"
expr_stmt   := call ";"
return_stmt := "return" expr ";"
loop_stmt   := "loop" LABEL "{" stmt* "}"
break_stmt  := "break" LABEL ";"
region_stmt := "region" REGIONID "{" stmt* "}"
check_stmt  := "check" expr "else" "trap" STRING ";"
give_stmt   := "give" expr ";"
match_stmt  := match_block
match_block := "match" expr "{" arm+ "}"
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

[GRAM-7] `match` has one arm shape (`{ stmt* }`, [GRAM-4]) and appears in two disjoint productions sharing one core-tree node kind [META-1]: as a statement (`match_stmt`) and as the initializer of a `let` (`let_stmt`, via `match_block`). Which production a given `match` parses under is production-local, not contextual (the [FORM-6] precedent). A `let`-initializer `match` is value-producing: on every control path each arm delivers the binding's declared `mode type` by terminating in one `give e;` [GIVE-1] or diverges (`return`/`break`/trap). A statement `match` produces no value; its arms act by effect and complete without one. `return`-position conditionals deliver by returning from arms; there is no helper-function conditional-initialization idiom, and value-production is confined to the `let`-initializer, so a `match` never occupies an arbitrary expression position.

[GIVE-1] `give e;` delivers `e` as the value of the arm of the nearest enclosing `let`-initializer `match`; `e` must have that `let`'s declared `mode type` (stated at the binder [TYPE-5], never inferred from arms). `give` is legal only inside a `let`-initializer `match` arm — a checker-scoped restriction exactly as `break`'s enclosing-loop rule [TYPE-6]: the grammar admits `give_stmt` and the checker restricts it, so `give`'s legality, not its meaning, depends on the enclosing construct, which is META-2-clean by the `break` precedent. On every control path a `let`-initializer `match` arm terminates in exactly one `give e;` or diverges; a give-free path, a statement following a `give` in the same block, and a second `give` on one path are each a hard error citing GIVE-1 — the value analog of match exhaustiveness [ERR-2]. Give-completeness is a structural last-statement recursion (an arm delivers when its final statement is `give`, `return`, `break`, or a nested value-`match` all of whose arms deliver), strictly simpler than the ownership checker. `give e;` moves or copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions exactly as a returned borrow of the same mode [OWN-4].

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

[TYPE-6] Name binding: declaration-before-use; a live name may not be shadowed or redeclared (one uniform rule for values, regions, labels); IDENT, REGIONID, and LABEL are disjoint namespaces; `break`'s LABEL must name a lexically enclosing loop. Enum variant constructor names are unique across all enum declarations in the program (the closed world, PROG-1), so a `construct` [GRAM-8] or `match` arm [GRAM-10] names its variant by a globally-unique constructor TYPEID, resolved without a type context.

[TYPE-7] Reading through a reference is explicit. `deref(place)` where place has type `&'r T`, `&uniq 'r T`, `box<T>`, or `arena<'r, T>` denotes a place of referent type T [GRAM-5]; a use of that place copies it when T is copy and requires `move` when T is affine [OWN-1]. A borrow-mode or box/arena binding used where a value of its referent type T is expected is a hard error citing TYPE-7, with the mechanical fix `deref(.)`. There is no implicit read-through-borrow [TYPE-4, META-2].

[CONST-1] A constant-expression (usable wherever the grammar's `const` non-terminal appears: `array<T, N>` sizes and `const` targs) is exactly one of: a decimal integer literal `[0-9]+` (bare, u64 by position); or an IDENT naming an in-scope integer-typed const-generic parameter [GRAM-2] or a top-level integer-typed named-const item [CONST-2]. The set is closed and total: no operators, no calls, no in-language computation in v0. Constant-expressions are evaluated at monomorphization [FN-2]. An IDENT resolving to a non-integer or array-typed const is a compile-time rejection [DIAG-1]. This closes the const-generic forwarding path: `const N` is usable as an `array<T, N>` size and forwardable as a `const` targ. Const arithmetic is DEFERRED with recorded delta; when added it carries a distinct const-eval overflow-policy name, does not overload the runtime `.trap` OPNAMEs, and is excluded from EFF-2's exhibits-traps relation.

[CONST-2] A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value. `type` must be const-eligible: a primitive [TYPE-1], or `array<T, N>` of const-eligible T; `box`, `buffer`, `arena`, and `slice` are not const-eligible (a const is pure static rodata: no allocation, no region, no drop). The `cvalue` totally defines the value (T1): a primitive-typed const takes a FORM-5 numeric or unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T. The const-dependency graph is acyclic and declaration-before-use [TYPE-6]; evaluation is substitution and layout only. A const item is never `move`d, `set`, or `&uniq`-borrowed. It is read via `index`/`len` (copy-out for copy elements) or shared-borrowed `&'r p` in any region [OWN-10], so a const table may be `slice_of`-viewed and passed to a consumer. Struct/enum-typed consts are DEFERRED with recorded delta.

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

An operation name is an OPNAME (dotted, closed mode-suffix, e.g. `iadd.checked`) or a dotless IDENT (`ieq ine ilt ile igt ige eeq ene feq flt fle band bor bxor bnot cvt len slice_of box_new arena_new`); both are consumed by `call` [GRAM-5] and resolved by name lookup: an OPNAME callee names its table op, and an IDENT callee names its table op where this table defines that spelling and a program `fn_decl` otherwise; a callee in neither is a hard error [DIAG-1]. The dotless operation IDENTs above and the mode-words `wrap` `trap` `checked` `sat` `strict` are RESERVED: no `fn_decl`, field, param, binder, or region binds them, which keeps op-vs-fn resolution context-free [META-2] and keeps a field-access place `p.field` from lexing as an OPNAME [FORM-3].

[OP-2] There are no wrap modes for division/remainder because no sound modular semantics exists for divisor-zero; this is table data, not an exception clause. (Negation has a wrap mode: two's-complement wrapping negation is sound modular arithmetic — ledger fix 2026-07-07.) Integer division and remainder have two checkable failures: a zero divisor for all int T, and, for signed T, the single signed-overflow case `iK::MIN / -1` (LLVM sdiv/srem are UB on both); `.trap` traps on either, and `.checked` returns `Err(DivideByZero())` for a zero divisor and `Err(DivOverflow())` for signed `iK::MIN / -1`, else `Ok`. DivOverflow is statically unreachable for unsigned T; the uniform `DivError` type is retained for regularity. Both classifications are table-fixed [ERR-4]. Mode-axis membership per family is table data: add/sub/mul carry {wrap, trap, checked, sat}; div/rem carry {trap, checked}; ineg and iabs carry {wrap, trap, checked}; shifts carry {wrap, trap}. Masking a shift amount discards writer intent, so a trap rung is offered; masking a rotate amount is the exact identity, so rotates are dotless-total [OP-8].

[OP-3] Float ops that ROUND carry `.strict` (IEEE 754, no reassociation, no contraction) and are the family a future fast-math mode would relax: `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` `fsqrt.strict` `ffma.strict`. Float ops that are EXACT or exact-selection are dotless: `fneg` `fabs` `fcopysign` `fmin` `fmax` `ffloor` `fceil` `ftrunc` `froundeven` `frem` and the six comparisons. Approximation/fast-math modes remain an OPEN numeric-semantics question; a relaxed float op would be introduced as a distinct OPNAME (FORM-1-additive).

[OP-4] `index<T>(p, i)` reads/writes are bounds-checked in all build modes when unproven; out-of-bounds traps [SCOPE-4]. "Proof" means deterministic-checker or verified-proof-artifact discharge; a solver may only promote performance-ledger facts and never licenses check elision. `index` applies to `array<T, N>`, `slice<'r, T>`, and `buffer<T>` places; a `buffer<T>` index is bounds-checked against the runtime length.

[OP-5] `check e else trap "msg";` is a runtime check in all build modes, never elided. A passed check creates the checked fact on the dominated path (stated-and-checked channel); the fuller stated-and-checked vocabulary (loop invariants, ranges) is DEFERRED with its delta.

[OP-6] cvt partition and semantics (cross-reference TYPE-4). `cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value is exactly representable in Dst (y the unique such Dst value) and `Err(NarrowError())` otherwise, and it never rounds, truncates, or saturates. A non-integral float-to-int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or infinity targeting an integer all yield `Err`; for float-to-float, an infinity maps to the same infinity and NaN maps to the target canonical quiet NaN (value-preserving). A pair is TOTAL — signature `(Src) -> own Dst`, no Result — where every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`. Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.

[OP-7] Operation-name convention (regularity, W1-predictable). An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix. `Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family. A `.mode` suffix appears iff the op sits on a mode axis, and single-behavior ops are dotless; the mode axes are the integer result-overflow axis {wrap, trap, checked, sat}, the shift out-of-range-amount axis {wrap, trap}, and the float rounding axis {strict}, with per-family membership fixed by [OP-2]. Signedness-parametric lowering keyed on the explicit type argument (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `ilt` = `slt`/`ult` row, not overloading. Nominal enum identity is likewise checked from the explicit type argument before `eeq`/`ene` lowering; equal representation width never makes distinct enum types interchangeable.

[OP-8] Edge semantics and confirmed lowerings for the operations added in this revision; every totality edge is closed here as table data, so no added row is writer-reachable poison (per T2 and W3). `iand`/`ior`/`ixor` lower to `and`/`or`/`xor` and `inot` to `xor x, -1` (total). A shift or rotate amount is `u32`; `ishl.wrap`/`ishr.wrap` mask the amount to `amt & (width-1)` and are total, `ishl.trap`/`ishr.trap` trap when `amt >= width`, `ishr` is `ashr` for signed T and `lshr` for unsigned T, and `irotl`/`irotr` lower to `llvm.fshl`/`llvm.fshr` whose amount is taken modulo width, so rotates are total. `ipopcount` is `llvm.ctpop`; `iclz`/`ictz` are `llvm.ctlz`/`llvm.cttz` with is-zero-poison false, so a zero input returns the bit width (the zero-input fix); counts return `u32`. `ibswap` is `llvm.bswap` (width a multiple of 16). `imulhi` is the high half of the full double-width product. `iadd.sat`/`isub.sat` are `llvm.sadd.sat`/`uadd.sat` or `ssub.sat`/`usub.sat` clamping to T's range; `imul.sat` widens, multiplies, and clamps, which avoids the signed-saturation miscompile in `llvm.smul.fix.sat`. `imin`/`imax` are `llvm.smin`/`umin` or `smax`/`umax`. `iabs.wrap`/`.trap`/`.checked` use `llvm.abs` with is-int-min-poison false, so `abs(iK::MIN)` is `iK::MIN` (the defined two's-complement edge value): `.wrap` returns it, `.trap` traps on it, and `.checked` returns `Err(Overflow())`. `reinterpret` is the LLVM bitcast instruction for cross-domain pairs (int<->float; bit-preserving, all NaN payloads and sign bits preserved) and an identity bit-relabel for same-width int<->int resign (i8<->u8, i16<->u16, i32<->u32, i64<->u64); it is the bit-preserving counterpart of value-preserving `cvt`, giving bit-level resign a home distinct from cvt's value-preserving resign. `fneg` is the LLVM fneg instruction (a sign-bit flip, not `fsub(0.0, x)`); `fabs` is `llvm.fabs`; `fcopysign` is `llvm.copysign`. `fmin`/`fmax` are `llvm.minimum`/`llvm.maximum` (IEEE-2019, NaN-propagating, negative zero ordered below positive zero, deterministic); `llvm.minnum`/`maxnum` are not used, because their signed-zero tie result is unspecified and breaks the reproducibility FORM-1 requires. `ffloor`/`fceil`/`ftrunc` are `llvm.floor`/`ceil`/`trunc` (roundToIntegral, staying in the float type); `froundeven` is `llvm.roundeven` (ties-to-even, matching `fadd.strict`). `frem` is the LLVM frem instruction (the C `fmod`: remainder with the dividend's sign, truncated quotient, exact), a distinct operation from IEEE `remainder`. `fsqrt.strict` is `llvm.sqrt` and `ffma.strict` is `llvm.fma` (single-rounding fused, distinct from the contraction [OP-3] forbids; a correctly-rounded libcall on hardware without an FMA unit). The comparisons `feq`/`flt`/`fle`/`fgt`/`fge` are ordered (`fcmp o*`, false when either operand is NaN) and `fne` is unordered (`fcmp une`), so `fne` equals `bnot(feq)` on every input and `fne(x, x)` is true exactly when x is NaN. `finf` is the positive-infinity value (negative infinity is `fneg(finf<T>())`) and `fnan` is the canonical quiet NaN; other NaN payloads are reachable through `reinterpret`. For a tag-only enum T, `eeq<T>(a, b)` is `True()` exactly when `a` and `b` denote the same declared variant of the same nominal T, and `ene<T>(a, b)` is its exact boolean complement. Both operands and the explicit type argument must have that exact T; representation equality never permits cross-enum comparison. `Bool` is admitted by the same tag-only rule. Both operations lower directly to equality or inequality of the validated discriminants in T's already-selected representation. They are pure and total: after normal operand evaluation, the primitive does not inspect a payload, access memory, trap, convert a value, or introduce a new optimizer fact channel; an operand read still exhibits its ordinary effect before the primitive executes. Payload-carrying enums, enum ordering, and enum/integer conversion remain outside the operation table.

[OP-9] `buffer_new<T>(n, v)` computes its allocation byte-size as `n * sizeof(T)` in u64 (sizeof(T) is a monomorphization-time constant). When this product overflows u64, `buffer_new` traps [SCOPE-4] before allocating: an unrepresentable buffer size is a contract violation, never a silent under-allocation (R4: no silent corruption; T2: no-UB), so `buffer_new`'s effect row includes `traps`. This is the sole allocation-size hazard `box_new`/`arena_new` (single-T, no runtime multiply) do not have. Allocation failure (OOM) is handled as by `box_new` (TCB-level, SCOPE-3), not a language trap. `array<T, N>` performs no runtime size computation (N is a constant-expression sized at monomorphization); a monomorphized array whose size exceeds the frame limit is a compile-time rejection [DIAG-1], so `array_new` is `pure`.

## 8. Functions, generics, contracts

[FN-1] Signatures state everything callers need: parameter modes/types, return mode/type, effect row, region parameters. Bodies are checked against signatures; callers rely only on signatures.

[FN-2] Generics are monomorphization-only; instantiation arguments are always explicit; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.

[FN-3] Contracts: a `contract` declares fn signatures and laws; `conform T : C { member = fn; }` declares conformance, checked per member; at most one conformance per (type, contract). The prelude marker contracts `Int` and `Float` [PRE-1] carry built-in closed conformer sets (`Int`: i8 i16 i32 i64 u8 u16 u32 u64; `Float`: f32 f64), not user `conform` declarations; a gparam bound `T: Int` (resp. `Float`) makes the integer (resp. float) operation-table rows [OP-1] and the identity literals `0_T`/`1_T` [FORM-5] available for `T`, monomorphized to the concrete type's ops. The exact predicate-vs-method encoding of built-in numeric contracts is a recorded refinement coupled to the generics layer.

[FN-4] Laws become optimizer-usable facts only via the stated-and-checked channel (static proof, runtime check under trap=abort, or verified proof artifact) or via gated-family ledger entries (§14). The law-test harness is non-normative prioritization for gate review; it never licenses optimizer use. LAWNAME is a closed table: `associative(f)`, `commutative(f)`, `identity(f, e)`.

[FN-5] No function values, no dynamic dispatch in the kernel. Behavior parameterization is generics over contract-conforming types (env-struct pattern); closed-set dispatch is `match`. Env-struct calls are guaranteed direct calls after monomorphization (never fn-pointer indirection). Typed operation tables and the mandated env-struct exactness diagnostics are DEFERRED constructs with recorded deltas.

[FN-6] Recursion is permitted. Polymorphic recursion is rejected by a syntactic rule: in any call cycle among generic functions, every call instantiates the callee at exactly the caller's own type parameters. This criterion is DELIBERATELY stronger than finiteness requires (it rejects some finite permutation cycles): predictable, locally explainable rejection per OWN-8's reject-and-restructure posture; the diagnostic must name the cycle and the restructuring. Rejection-rate measurement is a registered experiment.

[FN-7] Exactly one `fn main() -> unit` with effect row at most `allocates(heap), traps` must exist. There is no global state and no `'static` region in v0: ambient mutable globals would (a) erode the noalias fact base every function otherwise gets from parameter-only reachability (P0; carding backlog: GlobalsAA-class evidence), (b) create hidden inter-function channels invisible in signatures (W3, FN-1 signatures-as-trust-unit), and (c) pre-seed shared state for the future concurrency layer (T1). Immutable `const` items [CONST-2] are permitted and are not global mutable state: being read-only they never erode the noalias fact base (reads of frozen rodata add no aliasing hazard), create no hidden inter-function channel (the value is source-determined in the closed unit), and are Shareable-by-construction [CAP-1]; no `'static` region is introduced (borrows of const-rooted places obey the OWN-10 const clause), and there remains no writer-mutable global and no `static mut` analog.

[FN-8] A concrete `fn_decl` may carry one `requires` block after its effect row; `requires` is RESERVED and cannot bind any IDENT declaration. It is a checked callee-entry prologue, not an assumption and not a caller proof obligation: every invocation executes the block once after parameter binding and before the function body, including an invocation entering through a gated foreign boundary; a false final condition traps under [OP-5]/[EFF-4], and a true condition contributes its checked fact only to the dominated function body. Ordinary call acceptance never depends on proving the condition. The block contains zero or more `let_stmt` followed by exactly one final `check_stmt`. Its scope initially contains only the function parameters; each let introduces a fresh clause-local own copy value visible to later clause statements, and clause locals are not visible in the body. Every computation in the block must be an ANF [GRAM-9] call to a non-trapping, total operation-table row with effect `pure`; the final check condition is either a Bool clause atom or one such call returning Bool. User-function calls, construction, `move`, borrowing, `index`, mutation, control flow, allocation, and any trapping operation are rejected citing FN-8; a place is legal only as a non-consuming operand of an admitted table operation (for example `len<u8>(deref(out))`). Normal typing, ownership, and no-shadowing rules still apply. The final statement has exactly [OP-5] semantics; a deterministic proof from its passed fact may eliminate only downstream implicit checks such as [OP-4] bounds checks. `requires` is absent from `fn_sig` in this concrete-only first slice and cannot discharge a law under [FN-4]; contract/refinement support is DEFERRED with a recorded delta.

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

[PROG-1] One closed compilation unit; every name defined within it plus the prelude (§15). No separate compilation, no internal ABI, no dynamic loading, no reflection. The only boundary is the gated FFI wall (§14).

## 12. Diagnostics and artifacts (toolchain floor)

[DIAG-1] Every rejection cites exactly one rule ID, the node path in the canonical tree, and where applicable a mechanical fix or restructuring. Diagnostics are deterministic and byte-stable. A rejection whose node lies inside a nested `place` additionally renders the offending access-path segment (the specific `deref`/`index`/field step), not only the whole-place node path.

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
enum Bool { True(); False(); }

enum Option<T> { None(); Some(value: T); }

enum Result<T, E> { Ok(value: T); Err(error: E); }

enum Overflow { Overflow(); }

enum DivError { DivideByZero(); DivOverflow(); }

enum NarrowError { NarrowError(); }

contract Int {}

contract Float {}
```

## 16. Worked example (normative bytes)

[EX-1] The following complete program is byte-exact canonical form:

```
enum Sign { Neg(); Zero(); Pos(); }

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
