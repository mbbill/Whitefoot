# Kernel Specification v0.55

Status: ACTIVE v0.55
Prior versions: the immutable `spec/kernel-spec-vN.md` archives. These bytes are this version's identity; nothing else records it.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

## 1. Scope and conformance

[SCOPE-1] This document defines the complete writer-facing kernel. Values and functions have only the ordinary declaration, type, ownership, effect, and contract judgments defined here. Implementation language and linkage add no source-language category or permission.

[SCOPE-2] A program is checker-accepted iff it parses under the canonical grammar and satisfies every machine judgment in this document.
Every proof-required partial operation is statically discharged by the deterministic checker before lowering; a writer may expose a missing fact with executed control flow, provide a machine-proved loop-header or local invariant [INV-1], direct a larger local linear derivation with [PRF-1], or publish it across a function boundary through verified contracts [FN-8, FN-9].
Failure to discharge any such obligation rejects compilation; no operation receives an implicit runtime fallback and no writer statement can request one.
There is no writer-emittable unchecked state and nothing writer-stated is trusted without machine derivation.
Runtime-origin values — parameters, function results, loaded storage, and values derived from them — enter the proof context only as typed symbolic terms; their origin neither grants nor removes proof authority.
A proposition enters the context only from a selected ordinary control-flow edge, a declaration or type fact fixed by this specification, a verified callee postcondition, or the target of a machine-proved header or local invariant whose optional [PRF-1] certificate the checker has independently discharged.
No written conclusion, origin annotation, trusted-value mark, runtime observation outside executed source control, compiler-generated record, or optimizer result is a fact source.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on the declared trusted computing base: compiler, checker, linked function definitions, runtime, allocator, and OS.
Every linked definition must implement its ordinary declaration with the same value, ownership, effect, contract, and call-lifetime meaning as a Whitefoot body. Its implementation language and build binding do not alter source acceptance or permissions.
This version temporarily leaves only resource availability outside the source outcome model: heap exhaustion, stack exhaustion, operating-system quotas, and runtime-start resources may stop execution without a Whitefoot value, status, or cleanup guarantee.
That temporary scope cut does not defer static layout, stride, allocation-ceiling, address, target-domain, or parallel-independence proof; each obligation still succeeds before the governed operation is emitted.
Resource failure establishes no source fact, grants no fallback path, and cannot turn an unproved operation into an accepted one.

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

The left-attachment set contains `(`, `[`, `<`, `&`, `.`, `..`, and `::`.
The right-attachment set contains `)`, `]`, `>`, `,`, `;`, `.`, `:`, `(`, `<`, `[`, `..`, and `::`.
Between two consecutive terminals on the same line, emit zero bytes when the left terminal is in the left-attachment set or the right terminal is in the right-attachment set; otherwise emit exactly one ASCII space.
A `<` or `>` terminal selected by `compare_op` [GRAM-5] is rendered as a member of neither set, so a comparison is `a < b` while a type-argument list is `f::<T>(x)` and `buffer<u8>`; this stated spacing overrides the generic attachment of those two bytes exactly as the `for` header's stated space does below.
Thus function headers are `fn f()`, `fn f<T>()`, and `fn f['r](x: &'r i32)`; subscripts are `p[i]`; a counted range is `lower..upper`; generic and square-bracket interiors are compact; `](`, `>(`, and `::<` are attached; and commas and colons attach to their left operand and have one space before the grammar-required following element.
The colon separating an `actual_decl` name from its formal application is rendered as a member of neither attachment set: `actual SeedKey : Key<u64, Seed>`. This declaration separator is distinct from parameter, field, and bound colons, which retain the rule above.
Examples include `Result<i32, Overflow>`, `f(x: a, y: b)`, `cvt::<u8, u32>(w)`, `a <= b`, `actual SeedKey : Key<u64, Seed>`, `['r, 's]`, and `[10_u8, 20_u8]`.
The same rules render an elided region [FORM-8] with no further sentence: a borrow mode is `&u8` or `&uniq Foo`, a borrow expression is `&p` or `&uniq p`, a region-free view type is `Slice<u8>` or `arena<T>`, an unnamed region block opens `region {`, and a call whose region arguments are all determined writes no `::` application at all.

Every nonempty physical line begins with exactly two ASCII spaces for each enclosing brace block.
A closing brace is rendered after reducing the depth for the block it closes.
A match-arm header is therefore one level inside its match, and statements in the arm body are two levels inside it.

The line-bearing simple productions are `field`, `variant`, `fn_bind`, `const_decl`, `doc`, `contract_define`, `requires_clause`, `ensures_clause`, `set_stmt`, `expr_stmt`, `return_stmt`, `proof_use`, `break_stmt`, `give_stmt`, and `dispose_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs`, `propagate_let_rhs`, or `replace_let_rhs` and a `let_stmt` whose selected binder is a parenthesized binder list or a destructuring consume [GRAM-4, PROV-6].
Each renders completely on one line, including its final semicolon.
A `fn_sig` renders its signature inline, with a result-list space after `->` just as a `fn_decl` does. Its optional `contract_block` uses the ordinary block layout. In a formal body each member starts a new line and the following semicolon attaches to the signature or its contract's closing brace. In a `gparam` the signature stays in the surrounding generic header; no member semicolon is inserted.

The generically block-bearing productions are `struct_decl`, `enum_decl`, `formal_decl`, `actual_decl`, the body of `fn_decl`, `contract_block`, `region_stmt`, `match_stmt`, `value_match`, `if_stmt`, `value_if`, and `arm`.
Their introducer through `{` is one line; their children render on following lines at depth plus one; and `}` renders on its own line at the original depth.
Empty blocks still use an opening line followed by a closing-brace line.
An `invariant_stmt` ending in `;` renders completely on one line.
An `invariant_stmt` carrying a proof block renders its introducer through `{` on one line, each `proof_use` on a following line at depth plus one, and `}` on its own line at the original depth.

A `for_stmt` renders `for`, its optional label, exactly one space, and `(`; this stated space overrides the generic right attachment of `(`.
A `proof_use` whose `use_premise` is a delimited relation renders exactly one space before that premise's `(`, `use (a <= b);` and `use 3 times (a <= b);`; this stated space likewise overrides the generic right attachment of `(`, exactly as the `for_stmt` space above does, while the relation's own affine parentheses keep the generic attachment.
A `fn_decl` result list renders exactly one space between `->` and its `(`, a destructuring `let_stmt` exactly one space between `let` and its `(`, and a `set_stmt` target list exactly one space between `set` and its `(`; each of these three stated spaces overrides the generic right attachment of `(` exactly as the `for` header's does, so the canonical spellings are `-> (kept: own u64, spare: own u64)`, `let (kept, spare) = split(taken: move run);`, and `set (kept, spare) = split(taken: move run);` [GRAM-2, GRAM-4].
A `for_stmt` with no `header_invariant` renders its whole header, from `for` through `) {`, on one line; a counted loop with no invariant therefore has the one-line header `for (i in 0_u64..count) {`.
A `for_stmt` with at least one `header_invariant` breaks after `(` instead: its `for_binding` and every `header_invariant` each render on a separate following line at depth plus one, with a comma after every item except the last; and `) {` renders on one line at the original depth.
An ordinary `loop_stmt` without a parenthesized invariant header keeps the one-line introducer `loop` plus optional label through `{`.
With a header it instead renders `loop`, its optional label, exactly one space, and `(` on one line, again overriding generic right attachment; every `header_invariant` renders on a separate following line at depth plus one, with a comma after every item except the last; and `) {` renders on one line at the original depth.
In either loop form, body children and the final closing brace retain the ordinary block-bearing rendering.
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

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*` excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|defined|checked|sat|strict)` (single token; the base has the raw lowercase-word shape used by IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a valid field-access place `p.field`: all five suffix words are reserved from field binding [OP-1, GRAM-5]; e.g. `ineg.checked`).

[FORM-4] There are no comments.
Documentation is the `doc` field of declarations [GRAM-2].
Source coordinates and diagnostic derivations live in toolchain records.

[FORM-5] Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; a leading `-` is legal for signed TYPE, and the signed value must lie in TYPE's range [FORM-7]; e.g. `42_i32`, `-2147483648_i32`); finite floats use the grammar `-?(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE`, where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), positive exponents carry no sign, negative exponents carry one `-`, and only the integer and exponent components have the stated no-leading-zero form.
Let C be the nonnegative integer formed by concatenating the integer and fraction digits, let F be the number of fraction digits, and let E be the signed integer formed by the exponent digits and their optional `-`; when the exponent is absent E is zero, and `e-0` also gives E zero.
A matching decimal whose C is zero denotes signed decimal zero: a leading literal `-` selects negative zero and its absence selects positive zero, independently of E.
Every other matching decimal denotes the exact nonzero rational whose magnitude is C × 10^(E − F), with the leading literal sign applied.
For one finite bit pattern of TYPE, consider every matching decimal that rounds from that signed zero or exact nonzero rational to the bit pattern under IEEE 754 round-to-nearest, ties-to-even.
Its canonical spelling is the candidate with the fewest ASCII bytes before `_TYPE`; a tie is resolved by lexicographically least unsigned ASCII bytes.
This selection is total, host-independent, and unique; in particular `0.0` and `-0.0` remain distinct.
Other examples are `1.5_f64` and `6.022e23_f64`.
`unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, or one of exactly three escapes `\\ \" \n`; no other byte is legal, and each character has exactly one spelling (the escape where one is defined, the raw byte otherwise).
STRING appears only in `doc` entries; non-ASCII diagnostic text is DEFERRED.
There are no boolean literals: `Bool` is a prelude enum (§14).
Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric contract (`Int` or `Float`, §14), denoting T's additive and multiplicative identity; a concrete type uses `0_i32` and the like, so there is no dual spelling.
NaN and the infinities are not literals; they are the nullary ops `fnan` and `finf` [OP-1].

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual.
The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling follows the one-spelling convention [FORM-1] for the type's sole inhabitant.

[FORM-7] Numeric-literal well-formedness.
An integer literal `-?d_T` is legal where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` is legal for signed T, and `-0` is written `0`.
A float literal is legal only when it has the unique canonical spelling selected by [FORM-5] and denotes a finite value of its stated TYPE.
An out-of-range integer, a leading-zero integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value.

[FORM-8] Canonical region spelling.
A REGIONID is written exactly at the positions where this document does not otherwise fix the region denoted, and is absent at every other position, so each region position has exactly one legal spelling [FORM-1].
An absent REGIONID is neither a default nor a second meaning for a written one: the region is derived, in the class of the derived `let` binder mode [TYPE-5] and the derived match-binder mode [OWN-13], and an optional name whose absence resolves to the innermost enclosing construct is the unlabeled `break` form [TYPE-6] already carries [META-2].
Every clause below is decided by reading the owning declaration's own text, so a writer chooses the one legal spelling from the declaration alone and never from a checker verdict.
Being unnamed removes no obligation: an unnamed region has the ordinary extent, liveness, outlives, exclusivity, storage-duration, confinement, and loop judgments of the construct that introduces it [OWN-3, OWN-4, OWN-5, OWN-10, OWN-11, STOR-4].

The region positions of one `fn_decl` or `fn_sig` are its input positions — every REGIONID slot of its `param` list, in a `param`'s `mode` and at any depth of its `type` — and its output positions — every REGIONID slot of its `result_binding` `rtype` at any depth, and the REGIONID of each `arena` entry of an `allocates` effect [EFF-1]. An `allocates` entry over a store whose provider is a value carries a formal-rooted `effect_path` and no REGIONID.
`region_params` is a list of written names rather than a position, and a `reads` or `writes` `effect_path` names a place rather than a region [EFF-1], so neither is a position.
A region position is an invariant type position when it is a region argument of a source nominal [TYPE-2], a store-backed container, or a provider [PROV-1]; the region of a borrow mode or a view is a loan position instead, and the extent region of legacy `arena<'r, T>` is also a non-brand position.
This classification follows the declaration's explicitly written type constructors and type arguments at every depth, with their region slots resolved by PROV-1 where that rule fixes an elided brand, without inspecting a nominal's fields or expanding a generic type parameter into the type a caller supplies.
A formal region name is written at every invariant type position it occupies, even when it occupies only one position: a generic brand and [PROV-1]'s elided concrete store brand denote different types, not two spellings of one region.
At a non-brand position a region name is written exactly when the same region is meant at two or more positions of that same declaration, or when the position is an output position and that region is meant at no input position of that declaration.
Sharing one written name is the only way to relate positions; a region occupying no input position is the only region a caller must choose, because no actual argument determines it.
Every other non-brand position is unnamed and denotes a region distinct from the region of every other position of that declaration; elided store brands retain [PROV-1]'s meaning and are not fresh formal regions.
Every output position denoting a formal region therefore writes its region: either an input position names the same region, or none does and the caller supplies it.
An unnamed output non-brand position is a hard error citing FORM-8 at its `mode` or `type` production, because nothing in the declaration or at a call determines the region such a result carries; an elided concrete store brand fixed by PROV-1 is not an unnamed formal region and keeps that rule's meaning.
`region_params` is written exactly when at least one name is written by that judgment; it then lists exactly those names, once each, in the order of their first written occurrence in that declaration, and it is absent otherwise.
A declaration whose written names, name multiplicity, `region_params` membership, `region_params` order, or `region_params` presence differs from that rendering is a hard error citing FORM-8, using `SourceNode` at the owning `mode`, `type`, or `effect` production of the offending REGIONID, or at the complete `region_params` when the list itself is the defect.

A `borrow_expr` [GRAM-5] writes its REGIONID exactly when the region it denotes is not the region of the innermost region block lexically enclosing it; a `borrow_expr` no region block encloses therefore always writes it.
The enclosing region blocks are the `region_stmt`s enclosing it and the loop bodies enclosing it, because every `loop_stmt` and `for_stmt` body is itself a region block [OWN-11]; a borrow written directly in a loop body therefore takes that body's own per-iteration region and is written bare.
A `region_stmt` writes its REGIONID exactly when that name occurs at least once inside its body after this rule has been applied throughout that body, and is written `region { ... }` otherwise.
A loop body introduces its region with no REGIONID at all and no position can name it, so nothing outside that body reaches it.
An unnamed `region_stmt` introduces its region exactly as a named one does [OWN-3].
A written region the innermost enclosing region block already fixes, and an absent region at a `borrow_expr` no region block encloses, are each a hard error citing FORM-8 at that `borrow_expr`; an unreferenced written `region_stmt` name is a hard error citing FORM-8 at that `region_stmt`.

A `region_stmt` that is a loop body's only statement is a hard error citing FORM-8 at that `region_stmt`, whether or not it writes a name: its block is exactly that body, the body already introduces one region over that same block [OWN-11], and the two are therefore one region under two spellings [FORM-1].
A `region_stmt` the loop body writes any other statement beside is not that second spelling and stays legal, because its block is a strict part of the body and the two extents are distinguished: a bound borrow may end at that inner region before a later statement of the same iteration [OWN-4]. An unbound argument child's temporary loan ends at its [OWN-6] endpoint independently of that distinction; the loop body's own region is sufficient for its formation.
A writer therefore decides it by reading the loop body alone, asking only whether the body writes anything beside the block.
The one exception is a `region_stmt` some position inside its body must write its REGIONID at, which in a body is exactly a `targ` region argument [GRAM-5]: no implicit region has a name that position could carry, so such a block is the only spelling of its region and is admitted.
The mechanical repair is otherwise to delete the block, keep its statements as the loop body, and elide every REGIONID that named it.

A `call` whose callee resolves to an ordinary function [FN-1, FN-2] writes, as the leading region members of its `::` type application [GRAM-5], exactly those of the callee's region parameters that occupy no input position of the callee's declaration, in `region_params` order.
A call whose complete type application would then be empty writes no `::` at all.
Every other region parameter is determined by the call's own actual arguments and is not written.
For an input type, correspondence follows matching explicit type constructors, their declared region-argument positions in order, and their explicit type arguments recursively; a generic type parameter is one opaque type at this step, even when its actual contains brands.
The correspondence does not inspect nominal fields, choose a single region from a multi-region type, or require a region argument to occur at the outermost constructor.
When a formal region occupies any invariant input type position, the first corresponding actual brand fixes its substitution, every other invariant input occurrence must name exactly that brand [TYPE-5], and each actual region at a non-brand input position naming the same formal must outlive-or-equal that fixed region [OWN-4].
The fixed brand is never shortened to accommodate a loan, and permuting the parameter order does not change this judgment.
For a formal whose input occurrences are all non-brand positions, its substitution is that one of the actual regions at those positions which every actual region there outlives-or-equals [OWN-3]; a branded output adds no actual input brand and takes this substitution like every other output.
When no actual region at those non-brand positions has that property the call is a hard error citing OWN-4, exactly as an unsatisfiable written region argument is today; for loans the substituted region is otherwise the largest one every actual loan admits, so a related result region reaches as far as its inputs allow.
After determining all formal regions, complete argument types are checked against parameter types under that one final substitution [TYPE-5].
The outlives test does not adapt a view's type region: distinct `Slice` regions still require the explicit view-formation route [VIEW-2], not an implicit conversion [TYPE-4].
This substitution also preserves a legacy arena's extent and storage obligations [STOR-4], and grants no new lifetime conversion or cross-region move.
Writing a determined region parameter, or omitting an undetermined one, is a hard error citing FORM-8 at the complete `call`.
A retained-argument table operation still writes the region its row fixes, because no operand supplies it [TYPE-5, OP-1].

A constructor `call` [GRAM-5] of a nominal carrying `region_params` writes, as the leading region members of its `targs` list — which a construct writes bare, without the `::` a call writes — exactly those of that nominal's region parameters that the declared type of no field of the constructor it names determines, in `region_params` order.
A construct whose complete type application would then be empty writes no `targs` list at all.
A field determines a region parameter exactly when its explicitly declared type has a region slot denoting that region, including an elided brand PROV-1 fixes to the enclosing nominal's sole region parameter — the same constructor-by-constructor correspondence a parameter position bears at a call, including all region arguments and nested explicit type arguments — and an enum's variants are decided one by one, because a construct names one variant and writes exactly that variant's fields.
Every determined region parameter takes the region at its first corresponding actual position; every later occurrence, including one in the same field type, has the ordinary exact [TYPE-5] equality against the instance so determined and is never a second binding [PROV-1].
An operand whose type does not match the declared type at a determining position is a [TYPE-5] mismatch at that operand; the ordinary stored-content rules still decide which field types are admitted [STOR-5].
Writing a determined region parameter, or omitting an undetermined one, is a hard error citing FORM-8 at the complete constructor `call`, whose mechanical fix is `drop the region argument`.
This uses the same structural correspondence as the call clause above: a field operand stands where a parameter position stands, and construction still consults no expected nominal type [TYPE-5]; complete stored field types retain exact equality and are not shortened by a loan-region meet.

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
- `->`, `=>`, `..`, `==`, `!=`, `<=`, `>=`, and `::` are the eight compound punctuation tokens; each is formed exactly when its two bytes are adjacent, by the same maximal rule that forms `=>` from `=` and `>`.
The byte `!` occurs in no other token: a `!` not immediately followed by `=` is a raw lexical defect.
Otherwise each byte in `(`, `)`, `{`, `}`, `[`, `]`, `<`, `>`, `,`, `:`, `;`, `.`, `=`, and `&` is one exact punctuation token.

In source EBNF, each quoted fixed atom denotes the unique sequence of raw formed tokens whose concatenated bytes equal that atom.
In particular, `"&uniq"` expands to the punctuation token `&` followed by the fixed lower-word token `uniq`, while `"->"`, `"=>"`, `".."`, `"=="`, `"!="`, `"<="`, `">="`, and `"::"` each denote one compound punctuation token.
The quoted `"[0-9]+"` occurrences in the `const` production and the optional multiplicity position of `proof_use` share the grammar's sole pattern predicate: each denotes one numeric-form token whose complete bytes match `[0-9]+`, and neither is a fixed atom.
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
Every production maps 1:1 to one source-tree node kind.
The only abbreviation expansion is FN-3's hygienic expansion of formal and actual groups before semantic instantiation and IR; every expanded declaration and use retains its written source node and member position.
`infix_tail` maps to the `infix` node kind: a selected tail forms one `infix` node spanning the complete `expr` — the atom and the tail — so the 1:1 production-to-node mapping is preserved by the factored recognition; its operator child is one `infix_op` or one `compare_op` node.

[GRAM-2] Items:

```wf-ebnf GRAM-2
program      := item*
item         := fn_decl | struct_decl | enum_decl | formal_decl | actual_decl | const_decl
struct_decl  := "linear"? "struct" TYPEID generics? region_params? "{" doc? field* "}"
field        := IDENT ":" type ";"
enum_decl    := "linear"? "enum" TYPEID generics? region_params? "{" doc? variant* "}"
variant      := TYPEID "(" vfield_list? ")" ";"
vfield_list  := vfield ("," vfield)*
vfield       := IDENT ":" type
fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" ( result_binding | "(" result_binding ("," result_binding)+ ")" )
                effects contract_block? "{" doc? stmt* "}"
result_binding:= IDENT ":" rtype
contract_block:= "contract" "{" contract_define* requires_clause* ensures_clause* "}"
contract_define:= "define" IDENT "=" expr ";"
requires_clause:= "requires" clause_expr ";"
ensures_clause:= "ensures" ("when" result_route ":")? clause_expr ";"
result_route:= (IDENT "is")? TYPEID "(" fieldbind ")"
formal_decl  := "formal" TYPEID generics? "{" doc? (fn_sig ";")* "}"
actual_decl  := "actual" TYPEID region_params? ":" pack_use "{" doc? fn_bind* "}"
fn_sig       := "fn" IDENT region_params? "(" param_list? ")"
                "->" (result_binding | "(" result_binding ("," result_binding)+ ")")
                effects contract_block?
pack_use     := TYPEID targs?
function_arg := "fn" callee ("::" targs)?
const_decl   := "const" IDENT ":" type "=" cvalue ";"
fn_bind      := IDENT "=" callee ("::" targs)? ";"
doc          := "doc" STRING ";"
generics     := "<" gparam ("," gparam)* ">"
gparam       := TYPEID ":" (TYPEID | linearity_bound)
              | "const" IDENT ":" type | fn_sig | pack_use
region_params:= "[" region_param ("," region_param)* "]"
region_param := REGIONID (":" linearity_bound)?
linearity_bound:= "copy" | "affine" | "linear"
param_list   := param ("," param)*
param        := IDENT ":" mode type
```

[GRAM-3] Types and modes:

```wf-ebnf GRAM-3
type   := "i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|"f32"|"f64"|"unit"
        | TYPEID targs? | "array" "<" type "," const ">"
        | "Slice" "<" (REGIONID ",")? type ">"
        | "MutSlice" "<" (REGIONID ",")? type ">" | "box" "<" type ">"
        | "arena" "<" (REGIONID ",")? type ">" | "buffer" "<" type ">"
rtype  := mode type
mode   := "own" | "&" REGIONID? | "&uniq" REGIONID?
targs  := "<" targ ("," targ)* ">"
targ   := type | REGIONID | const | function_arg
```

[GRAM-4] Statements:

```wf-ebnf GRAM-4
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | for_stmt | invariant_stmt | break_stmt | region_stmt
             | if_stmt | match_stmt | give_stmt | dispose_stmt
let_stmt    := "let" ( IDENT "="
               ( ordinary_let_rhs | propagate_let_rhs | replace_let_rhs
               | value_match | value_if )
               | "(" IDENT ("," IDENT)+ ")" "=" call ";"
               | TYPEID "(" fieldbind_list? ")" "=" "move" place ";" )
if_stmt     := "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
value_if    := "if" expr "{" stmt* "}" "else" (value_if | "{" stmt* "}")
ordinary_let_rhs:= expr ";"
propagate_let_rhs := "propagate" expr ";"
replace_let_rhs := "replace" place "=" expr ";"
set_stmt    := "set" ( place "=" expr ";"
               | "(" place ("," place)+ ")" "=" expr ("," expr)* ";" )
expr_stmt   := call ";"
return_stmt := "return" expr ("," expr)* ";"
loop_stmt   := "loop" LABEL? ("(" header_invariant ("," header_invariant)* ")")?
               "{" stmt* "}"
for_stmt    := "for" LABEL? "(" for_binding ("," header_invariant)* ")"
               "{" stmt* "}"
for_binding := IDENT "in" atom ".." atom
header_invariant := "invariant" IDENT ":" affine_expr compare_op affine_expr
invariant_stmt := "invariant" IDENT ":" affine_expr compare_op affine_expr
                  (";" | "{" proof_use+ "}")
proof_use   := "use" (("[0-9]+" | IDENT) "times")? use_premise ";"
use_premise := IDENT | "(" affine_expr compare_op affine_expr ")"
affine_expr := affine_term (affine_add_op affine_term)*
affine_term := affine_factor ("*" affine_factor)?
affine_factor := atom | call | "(" affine_expr ")"
affine_add_op := "+" | "-"
break_stmt  := "break" LABEL? ";"
region_stmt := "region" REGIONID? "{" stmt* "}"
give_stmt   := "give" expr ";"
dispose_stmt:= "dispose" place ";"
match_stmt  := "match" expr "{" arm+ "}"
value_match := "match" expr "{" arm+ "}"
arm            := TYPEID "(" fieldbind_list? ")" "=>" "{" stmt* "}"
fieldbind_list := fieldbind ("," fieldbind)*
fieldbind      := IDENT ":" IDENT
```

[GRAM-5] Expressions and places:

```wf-ebnf GRAM-5
expr           := atom infix_tail? | call
infix_tail     := (infix_op | compare_op) atom
infix_op       := "+" | "+wrap" | "+defined" | "+checked" | "+sat"
                | "-" | "-wrap" | "-defined" | "-checked" | "-sat"
                | "*" | "*wrap" | "*defined" | "*checked" | "*sat"
                | "/" | "/defined" | "/checked"
                | "%" | "%defined" | "%checked"
compare_op     := "==" | "!=" | "<" | "<=" | ">" | ">="
atom           := literal | "move" place | place | borrow_expr
call           := callee ("::" targs)? "(" ( atom_list | fieldinit_list )? ")"
callee         := IDENT | OPNAME | pack_use ("::" IDENT)?
fieldinit_list := fieldinit ("," fieldinit)*
fieldinit      := IDENT ":" atom
borrow_expr    := "&" REGIONID? place | "&uniq" REGIONID? place
atom_list      := atom ("," atom)*
clause_expr    := affine_expr (clause_op affine_expr)?
clause_op      := compare_op | "+defined" | "-defined" | "*defined"
                | "/defined" | "%defined"
place          := pbase psuffix*
pbase          := IDENT | "deref" "(" place ")" | "entry" "(" IDENT ")"
psuffix        := "." IDENT | "[" atom "]"
```

[GRAM-6] There is no general operator syntax and no precedence: an `infix` expression is exactly one operation over two atoms [GRAM-5, GRAM-9], composition is by `let`, and no precedence, associativity, or parenthesization surface exists.
The `compare_op` alternatives are the six integer comparisons of [OP-1] and form `infix` expressions exactly as the `infix_op` arithmetic does; a `call` writes its type and region arguments after the `::` delimiter, `cvt::<u8, u32>(w)`, so that `IDENT "<"` begins a comparison and never a type-argument list, while a constructor `call` and a `type` write theirs bare.
There is no `while`.
Conditional control is type-driven with one form per class: a Bool condition takes `if`/`else`, an enum scrutinee takes `match`, and each is the sole legal form for its class — a `match` whose scrutinee has type `Bool` is a hard error citing GRAM-6 at the scrutinee `expr` node (spell `if`).
An `if` condition must have exact value mode and type `own Bool` under exactly the [OP-5] condition judgment, TYPE-7 exclusivity included; every other condition failure cites GRAM-6 at the condition `expr` node.
An `if_stmt` `else` whose block is empty is a hard error citing GRAM-6 at that `if_stmt` node (spell the else-free `if`; a `value_if`'s undelivering else is [GIVE-1]'s rejection, not this one).
An `else` whose block contains exactly one `if_stmt` and nothing else is a hard error citing GRAM-6 at that nested `if_stmt` node (spell `else if`); in a `value_if` whose else block is exactly one else-free `if_stmt`, the branch cannot deliver, [GIVE-1] owns the rejection, and GRAM-6 forms no candidate there, so the flattening fix is never demanded where the chain form could not be spelled.
A conditional value is a `let`-initializer `match` or `if` [GRAM-7, GIVE-1].
The only iteration forms are the ordinary `loop` plus `break`, and the counted ascending half-open `for` form whose complete semantics are [TYPE-5, TYPE-6, OWN-11, FN-1, ENT-2, ENT-3, ENT-5]; there is no step, reverse, iterator, or `continue` form.
The subscript suffix is a place form (its sole home); bounds semantics are [OP-4].
A `clause_expr` is the contract-clause shape and its sole home is a `requires_clause` and an `ensures_clause` [GRAM-2]: one `affine_expr`, or two around one `clause_op`.
Its side is [GRAM-4]'s own `affine_expr` [INV-1] and its factor is an `atom`, a `call`, or a constructor `call`, which is what lets a contract clause name a measure of a place on either side of its comparison and displace it by an affine expression [MSR-5].
`+`, `-`, and `*` are consumed inside `affine_expr` and are never a `clause_op`, so the operator position carries exactly the Bool-valued rows of [OP-1] — the six comparisons and the five infix `defined` domain queries — and no alternative of the union derives two ways [GRAM-1].
Every other position keeps [GRAM-9]'s one-operation-over-two-atoms shape and a nested call is still bound by its own `let`.

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
A call with a normal result edge does not itself count as delivery or must-divergence.
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
A constructor `call` of struct or enum-variant type K writes every declared field of K exactly once as `IDENT ":" atom`, the IDENTs equal to K's declared field names in declared order.
A missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-8 and K's declared field list.
There is no positional construction form; a nullary K is written `K()`.
Field names are redundant-explicit facts (the TYPE-5 class): checked, never chosen, never a reordering option (declared order is the one legal byte sequence).
The name-only-when-two-same-typed-fields alternative is a context-dependent spelling and is rejected [META-2].

[GRAM-9] Flat (three-address) computation.
Every call argument, construct field value, infix operand, subscript offset, and lower or upper endpoint of a `for_stmt` is an `atom` [GRAM-5]; a `call` or constructor `call` in an atom position does not derive under the grammar and is a hard error citing GRAM-9.
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
A `call` whose callee resolves to an ordinary function writes its arguments as `fieldinit_list` [GRAM-5] — each `IDENT ":" atom` equal to the callee's declared parameter names in declared order [FN-1], the GRAM-8 discipline applied to calls.
A missing, extra, repeated, misspelled, or out-of-order parameter name is a hard error citing GRAM-11 and the callee's parameter list.
A `call` whose callee resolves to a table operation [OP-1] writes positional `atom_list` operands (operands are order-intrinsic and unnamed).
Argument reordering is not a spelling option: declared order is the one legal byte sequence [FORM-1], so parameter names are redundant checked facts, never a reordering license.
Callee kind is resolved by name lookup [OP-1], the same partition that already selects the callee.

## 4. Types

[TYPE-1] Primitive types: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 unit`.
(`Bool` is a prelude enum, §14, not a primitive.)

[TYPE-2] Composite types: `struct`, `enum`, `array<T, N>` (N a constant-expression, [CONST-1]), `Slice<'r, T>` and `MutSlice<'r, T>` (the two region-carrying views [VIEW-1]), `box<T>` (heap-owned unique), `arena<'r, T>` (region-bounded owned), `buffer<T>` (heap-owned, runtime-length, flat contiguous {data-pointer, u64 length} value; affine single-owner; length fixed at allocation, no in-place growth).
Five further composite types are compiler-owned nominals of the nominal-type TYPEID domain [TYPE-6], written as a TYPEID with `targs` [GRAM-3] and declared by no source item: the two runs `FixedVector<T, n>` (`n` a constant-expression, [CONST-1]) and `Vector<'s, T>` [BLK-1], the two providers `Heap<'s>` and `Arena<'s, bytes, align>` (`bytes` and `align` constant-expressions) [PROV-1], and the cell `Box<'s, T>` [S39].
`Box<'s, T>` is one value of `T` resident in the store `'s` names, store-branded on exactly [PROV-1]'s terms: its region is a component of its type, its brand resolves by that rule, and its release class is read off that region alone [PROV-6].
It carries **no measure at all** — a cell is never empty, so [MSR-1]'s table gives it no row and a read of one owes no proof — and its referent is any nameable type [TYPE-3], including one that reaches the cell's own nominal, which is how a recursive type is written [PROV-6].
`box<T>` is the ambient-heap cell this one replaces at every store a program holds as a value; the two coexist while `box<T>` lives.
A run's element type is any nameable type and neither run inherits this rule's element restriction on `buffer` [BLK-1]; a provider has no writer-visible component, no literal, and no source construction route; a general provider enters a function as an ordinary parameter [PROG-3], and an extent provider is produced by a reserving operation [BLK-2].
Every value of one of the five is affine [OWN-1], and each is region-bearing under [STOR-5]'s relation exactly when a move of it would strand or hide something: the two providers always are, and `Vector<'s, T>`, `Box<'s, T>` and `FixedVector<T, n>` never are, a store-branded value's brand confining the position it occupies rather than hiding a provenance [PROV-1].
A `struct` or `enum` declaration may carry the `linear` modifier [GRAM-2], which states a logical must-consume obligation on values of that nominal in every scope and changes no component, layout, or construction route [PROV-6].
A `struct` or `enum` declaration may also carry `region_params` [GRAM-2]: the declared regions are components of the nominal's type name, are invariant [OWN-12, TYPE-5], and are the regions [PROV-1]'s first brand-resolution clause reads at a stored position of that nominal.
Being a component of the name is the whole of their meaning: an instance of such a nominal is fixed by its region arguments beside its type and const arguments, two instances of one declaration whose region arguments differ are two types under the exact identity [OWN-12] and [TYPE-5] already perform, and every position of the declaration that names a region parameter carries that instance's own argument — so a field of type `Vector<'s, T>` in an instance at `'a` has type `Vector<'a, T>` and takes the release class `'a`'s own declaration gives it [PROV-6].
A nominal's region arguments are written as the leading members of its `targs` [GRAM-3], where the two runs and the two providers already write theirs, in `region_params` order, at every `type` position and at every constructor `call` [TYPE-5].
An opaque nominal [PRE-1] has no writer-visible component or constructor. It obeys the ordinary nominal type, ownership, and call rules.
An `array<T, N>` owns exactly N initialized values of T in ascending index order, where T is any nameable type [TYPE-3], copy, affine, or linear [OWN-1, PROV-6], subject to the ordinary stored-content and generic-argument judgments [STOR-5, FN-2].
It is a complete value, with no vacant element and no mutable window: its four measures are [MSR-1]'s type-fixed row, and its dense inline representation is [STOR-1]'s.
Its construction routes are the copy-fill operation `array_new` [OP-1], a const initializer when [CONST-2] admits it, and the consuming conversion [BLK-3]; type formation does not widen any constructor's own domain.
Its element places obey the ordinary indexing, borrowing, replacement, and same-statement read-out judgments [OP-4, OWN-5, SET-2, LIV-2]; none admits an uninitialized element in a live array.
A `buffer` element type T must be copy or a region-free [STOR-5] affine type; construction is gated per operation — `buffer_new` fills only copy elements, and `buffer_vacant` constructs `Option`-element buffers [OP-1, OP-9] — so an affine-element buffer type outside those constructors is well-formed but has no v0 construction route, exactly the formation/construction distinction this rule already draws for its element domains.
Affine elements leave and enter their slots through [SET-2] element replacement and through the [LIV-2] read-out of an element target of the same `set`, and are read in place through borrowed `match` [OWN-13]; neither exchange changes the buffer's length [ENT-5].

[TYPE-3] Nameability: every constructible type/mode/effect has a canonical, finite, writable name requiring no compiler execution.
The `linear` modifier and a generic parameter's linearity bound are properties of a declaration and not components of a type name: two instances of one nominal have one name whether or not its declaration is marked, and no name spells a linearity class [PROV-6].

[TYPE-4] There are no implicit conversions.
Numeric value conversion is the single explicit op `cvt::<Src, Dst>(x)`.
Totality is decided by value-preservation, not bit-width: `cvt` returns `own Dst` where every value of Src is exactly representable in Dst, and `own Result<Dst, NarrowError>` for every other distinct numeric pair; it never rounds, truncates, or saturates.
The exact partition and per-value semantics are [OP-6].
Deliberate rounding is a separate DEFERRED float-round op family, never `cvt`.

[TYPE-5] Statement-local typing; boundary-explicit facts.
The factored `call` grammar denotes a construction exactly when its callee is an unqualified TYPEID application. A constructor writes any nominal arguments directly after that TYPEID, never with the function-call `::` introducer; writing the latter is a TYPE-5 error at the complete call. Its operands are named fields under GRAM-8, so a positional operand list is a GRAM-8 error there. Construction is an ordinary expression, not the callable occurrence required by an expression statement or a destructuring result-list let; either statement position rejects it under TYPE-5. These judgments preserve the constructor forms while sharing the strong-LL(2) prefix with qualified member calls.
A `let` binder's mode and type are derived, never written: exactly the mode and type its selected right-hand side produces — an `ordinary_let_rhs` from its expression, which is always self-typed (operands are typed atoms, calls are typed by their [FN-1]/[OP-1] signatures, literals carry mandatory suffixes [FORM-5], constructions name their nominal and, when that nominal is generic, write its arguments); a `propagate_let_rhs` from the propagated Ok payload [ERR-3]; a `replace_let_rhs` at mode `own` from its target place's final selected type [SET-2]; a `value_match` or `value_if` from the derived common delivery type [GIVE-1], whose delivering `give`s are inside the same `let_stmt`, so the derivation stays statement-local; and a parenthesized binder list from its `call`'s declared result ordinals, binder i at result ordinal i's written mode and type [GRAM-4, FN-1, CALL-4].
This is unique reconstruction, not inference: no binder's type depends on a later statement, an expected type, or any use site, and no two derivations can disagree [FORM-1].
One form is excluded rather than reconstructed: a body `let` may not annotate a borrow with a region its right-hand side did not name, stating a destination the right-hand side satisfies by outlives [OWN-4] rather than equals, and a derived type is always the region the right-hand side itself produces.
Call sites state explicitly exactly what their callee class requires: type, const, and function arguments for user generics [FN-2], including group abbreviations; the region arguments [FORM-8] leaves undetermined for an ordinary function; for a kernel-domain operation [BLK-0], each type, const, and region argument no operand of its row supplies, decided per argument rather than per callee; and, for exactly the retained-argument table operations — `cvt` and `reinterpret` (type pairs [OP-6, OP-8]), `array_new` (element type and const length [CONST-1]), `arena_new` (region and element type), `buffer_fits` and `buffer_vacant` (element type [OP-1, OP-9]), and `finf`/`fnan` (result type) — the written arguments their rows fix, because no operand can supply them.
A constructor `call` of a generic nominal states that nominal's type, const, and function arguments on the same ground and in every position, mandatorily: the source nominals under [FN-2], and the prelude generic nominals `Option<T>` and `Result<T, E>` through their variant constructors `None`, `Some`, `Ok`, and `Err`.
A constructor `call` of a nominal carrying `region_params` states, as the leading members of the same list, exactly those of its region parameters that no field of the constructor it names determines [FORM-8], and on exactly that ground: construction consults no expected nominal type, so the field operands and the written members are the only supply there is, and a region argument is a component of the instance's name [TYPE-2].
This is not a second region-spelling rule: [FORM-8] writes a REGIONID wherever this document does not otherwise fix the region denoted, and a `type` position fixes none of a nominal's region arguments while a constructor `call`'s own field operands fix exactly the ones their declared types name.
A parameter type determines each formal region in its explicitly written type tree from the corresponding position of its actual [FORM-8], including every region argument of a multi-region nominal and regions in nested explicit type arguments; an opaque generic type parameter supplies no additional formal region positions.
A nullary `None()` has no operand to supply anything, and construction never consults an expected nominal type [TYPE-6], so the written arguments are the only supply there is; their absence, or a count other than the named nominal's parameter list, is a hard error citing TYPE-5 at the complete constructor `call`.
A non-generic prelude nominal [PRE-1] has no parameters and writes no type arguments.
Every other table operation carries no written argument and derives its selected type from its operands [OP-2]; a written argument there is a hard error citing OP-1.
Argument types match declared parameter types exactly.
After [SET-1] derives a writable target place of type T, the right-hand side of `set p = e;` must produce exactly `own T`; there is no mode coercion, type conversion, or target-selected operation overload.
After the TYPE-7 implicit-read exclusivity below, a different right-hand-side mode or type is a hard error citing TYPE-5 at the complete `expr` child of the `set_stmt`, carrying expected `own T` and the actual mode and type.
After [SET-2] derives a writable affine target place of type T, the right-hand side of `let x = replace p = e;` receives this same exact-`own T` judgment, located at the complete `expr` child of the `replace_let_rhs`.
A `set` target list receives the same judgment once per target: after [SET-1] derives writable target i of type T, ordinal i of the right-hand side — result ordinal i of its one `call`, or written expression i of its value list — must be exactly `own T` [GRAM-4, CALL-4, LIV-2].
A binder list whose written count differs from its `call`'s declared result count, or whose callee declares a single result, and a target list over one `call` in the same two states, are a hard error citing TYPE-5 at the complete `call` child of the statement, carrying the written count and the actual result.
A target list whose written value list has a different length from its target list is the same rejection, located at the complete `set_stmt`.
Redundant-explicit facts remain mandatory at every trust boundary — signatures with full modes, types, and effect rows [FN-1], construction field names [GRAM-8], match binders [GRAM-10], call argument names [GRAM-11] — and are deleted exactly where reconstruction is unique and no transposition risk exists.
A written region is such a fact where it names an invariant generic brand, relates two positions, or names a region the caller must choose; [FORM-8] deletes a lone input loan name, while preserving [PROV-1]'s distinction between a named generic brand and an elided concrete store.

Every result ordinal of a `fn_decl` or `fn_sig` has one mandatory `result_binding` whose written `rtype` fixes that callable result's mode and type.
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
| lexical IDENT | top-level `fn_decl`; raw function-kind `gparam`; top-level `const_decl`; const `gparam`; `param`; `let_stmt`; `for_stmt` binder; arm `fieldbind` binders; `contract_define`; FN-9-owned result and route candidates; PRE-1 functions; admitted kernel-domain operations [BLK-0] | a `callee` IDENT admits a top-level function, in-scope function parameter, PRE-1 function, or admitted kernel-domain operation; an unqualified `function_arg` or `fn_bind` right side admits an ordinary function or function parameter; `const` IDENT admits an in-scope const generic or earlier named const; `cvalue` IDENT admits an earlier named const; `pbase` admits an in-scope runtime value binding, contract definition, admitted symbolic result datum, named const, or in-scope const generic [MSR-6] |
| nominal-type TYPEID | source `struct_decl` and `enum_decl` names; source formal and actual groups; PRE-1 nominal types; the five compiler-owned container and provider nominals [TYPE-2]; lexical type `gparam`s overlay this domain while live | a runtime `type` or generic-numeric suffix admits only its ordinary type class; an explicit `targ` additionally admits a formal or actual abbreviation; a `pack_use` admits a formal or actual group, with FN-3/FN-5 checking its position and member selection |
| constructor TYPEID | each source struct constructor under its struct TYPEID; every source enum `variant`; PRE-1 variants, classified as struct-constructor or enum-variant; PRE-1 struct constructors | the leading TYPEID of constructor `call` admits either class; the leading TYPEID of `arm` or `result_route` admits only enum-variant |
| numeric-bound TYPEID | the two built-in bounds `Int` and `Float` [PRE-1] | the bound TYPEID of a type `gparam`; a linearity bound instead uses its fixed grammar spelling [GRAM-2, PROV-6] |
| REGIONID | `region_params` of a `fn_decl`, `fn_sig`, `actual_decl`, `struct_decl`, or `enum_decl`, and a named `region_stmt` | every written REGIONID in `type`, `mode`, `targ`, arena-allocation effects, and `borrow_expr` [FORM-8]; an actual header captures only the store brands its formal type arguments name [FN-3] |
| LABEL | an optional LABEL written by `loop_stmt` or `for_stmt` | an optional LABEL written by `break_stmt` |
| invariant IDENT | names written by `header_invariant` and `invariant_stmt` | the IDENT premise alternative of `use_premise` |

A source struct contributes one declaration event that adds one nominal-type entry and one constructor entry with the same spelling.
Those entries do not collide because the grammar distinguishes a `type` role from a constructor `call` or `arm` role.
An enum declaration adds only its nominal type; each variant adds its constructor.
Entries must be unique within, but not across, the nominal-type, constructor, and numeric-bound domains. Formal and actual group names share the nominal-type collision domain, but neither is a runtime type or a constructor.
Constructor uniqueness is whole-unit and context-free, so construction and matching never consult an expected nominal type.

PRE-1 contributes its declaration records in the preorder stated there.
The prelude's nominals, constructors, functions and numeric bounds enter the ordinary whole-unit lookup inventory and are visible throughout the closed unit. A declaration's type and region parameters, value parameters and fields are owner-local and enter only that declaration's ordinary owner tables.
PRE-1 records have no source event or source node.
Every top-level function signature is visible throughout the closed compilation unit after unit formation and before any semantic use is resolved [FN-1].
A source nominal type, formal group, or actual group becomes visible immediately after its declaring TYPEID terminal.
A source struct constructor becomes visible at that same terminal; an enum-variant constructor becomes visible immediately after its variant TYPEID terminal.
Each remains visible through the end of the unit.
Whole-unit inventory checks uniqueness but grants no earlier visibility; a use before one of these declaration points is rejected even though inventory knows the later declaration exists.

A generic TYPEID parameter becomes visible after its declaring terminal through the remainder of its declaration's generic, header, and body scope.
It may not redeclare another parameter in the same generic list or shadow a live nominal type or enclosing generic type.
Constructor and numeric-bound spellings are separate grammar-selected domains and do not participate in that comparison. Formal and actual names do participate.
A const generic becomes visible after its complete `gparam`. A raw function-kind parameter becomes visible after its complete `fn_sig` through the receiving declaration's remaining header and body; its own value parameters and proof candidates remain local to its signature. A formal-group member has a declaration identity but no unqualified lexical entry; FN-5 selects it through the written group application.
A region parameter becomes visible after its terminal through the remainder of its signature and body; an actual's region parameters are visible through its formal application and bindings; for `fn_sig`, that scope ends at the signature terminator; for a `struct_decl` or an `enum_decl`, it is visible through the remainder of that declaration and nowhere else, and every written use of that nominal supplies one region argument per declared region parameter in declared order [TYPE-2, FN-2].
Independently of visibility, OWN-3 requires every REGIONID declaration to be unique throughout its owning function declaration or function-formal signature, parameters included: a later region parameter or local region may not reuse an earlier region spelling even after the earlier region's lexical scope has ended.
A `fn_decl` parameter becomes visible after its complete `param` through the function's optional `contract_block` and body.
A `fn_sig` parameter becomes visible after its complete `param` through that signature's effects and optional contract block; duplicate parameters in that signature are same-scope redeclarations. It is not visible in a sibling member or the receiving function's body.
A `let_stmt` binder becomes visible only after its complete initializer statement through the end of its lexical block.
A `contract_define` binder becomes visible after its complete initializer through later definitions and every following requires or ensures clause of that one block, and nowhere in the body.
Its initializer may therefore use parameters, named consts, live type or const parameters, and earlier definitions, never itself or a later definition.
The IDENT of every `result_binding` is an FN-9-owned proof candidate rather than a runtime TYPE-6 value declaration.
It participates in FORM-3 reservation and must differ from every parameter and definition in its function or function-formal signature.
In a `fn_decl` or `fn_sig`, FN-9 admits each eligible ordinal as a symbolic datum visible only in that signature's ensures clauses under the same result-route rules.
The second IDENT of a `result_route` fieldbind is an FN-9-owned payload candidate.
After the route's leading constructor and field are admitted, that binder is visible only in the same `ensures_clause` expression and the header result binder is not visible there.
It must differ from its paired field, every parameter, the result binder, and every live definition.
Different ensures clauses have disjoint result-datum scopes and may reuse one route-binder spelling.
Neither kind of result datum has runtime storage or ownership state, and neither is visible in the function body.
A match binder becomes visible in its arm body only after the complete fieldbind list and only after GRAM-10 has established that it differs from its paired field label, every earlier binder in that arm list, and every lexical-IDENT declaration live on arm entry.
A `for_binding` binder becomes visible after its complete `for_binding`, including both endpoint atoms, through the remaining `header_invariant` clauses and the counted body; it is not visible in either endpoint.
An ordinary or counted loop label, when written, and a local region are visible only in their respective bodies; a counted label is not visible in the binding or invariant header.
A loop label is an optional lexical name, never the identity of the loop: every `loop_stmt` and `for_stmt` has one distinct compiler-owned structural loop identity whether or not it writes a LABEL.
An unlabeled `break;` must be lexically inside at least one ordinary or counted loop and resolves to the nearest such enclosing loop.
A labeled `break @name;` performs the ordinary LABEL-domain lookup below and may therefore resolve past one or more inner loops to an enclosing loop carrying that spelling.
The resolved loop's structural identity, not a LABEL declaration, is the target retained by the semantic checker.
A `header_invariant` name is a proof-only declaration in a separate invariant-name domain.
All names in one header must be distinct; none is visible in the header itself or before the loop, and after the complete header all become visible simultaneously throughout that loop body only.
An `invariant_stmt` name becomes visible only after its complete statement through the remainder of its lexical block and nested blocks.
An invariant name never denotes a runtime value, place, ownership object, label, or callable, and it is referenced only by the IDENT premise alternative of `use_premise` under [PRF-1].
Within the invariant-name domain a new live declaration may not shadow another live declaration, while disjoint expired scopes may reuse a spelling.
Adding, removing, or changing a loop label cannot change any invariant binding.
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

The owner-dependent declaration and use roles are exactly the carriers classified by [DIAG-1].
They do not enter or query a lexical name domain.
DIAG-1 retains each owner-dependent carrier for later typed owner/member checking.
Deferral is neither acceptance nor rejection of its later owner/member relation.

[MSR-6] A const generic is a value wherever a named const is.
The `pbase` admission of [TYPE-6] carries an in-scope const generic, and with it the `for_stmt` endpoint admission of [ENT-2], the clause operand of [MSR-5], and the affine atom of [INV-1] carry one too.
The first three are the positions in which a named const is already a value; the fourth is one it is not, and a const generic is admitted there on its own ground: [ENT-2] clause (c) makes it a constant rather than a tracked place, so it needs no liveness, no entry state and no support, while a named const is a term of clause (a) whose exclusion from the affine atom this version keeps.
A read of an in-scope const generic is a `pbase` with no `psuffix` and no `deref` wrapping; its exact type is the `gparam`'s written integer type and its value mode is exact `own`, so the ordinary [TYPE-5] check applies at each use with no widening of any other judgment.
Reading a const generic performs no operation, allocates nothing, and has the empty effect row.
A const generic is fixed at [FN-2] instantiation, so a concrete instance reads a mathematical constant and the one source-canonical symbolic instance reads the symbolic constant term [ENT-2] clause (c) already fixes; this rule adds a spelling and no fact source.
It introduces no shadowing hazard, because a const generic is already a lexical-IDENT declaration and a colliding later binding is a TYPE-6 redeclaration before this admission is consulted.
Absence of an in-scope const generic under this spelling remains the ordinary [TYPE-5] unresolved-use rejection.

[TYPE-7] Reading through a reference is explicit.
`deref(place)` where place has type `&'r T`, `&uniq 'r T`, `box<T>`, `Box<'s, T>` [S39], or `arena<'r, T>` denotes a place of referent type T [GRAM-5]; a use of that place copies it when T is copy and requires `move` when T is affine [OWN-1].
A borrow-mode, cell, or arena binding used where a value of its referent type T is expected is a hard error citing TYPE-7, with the mechanical fix `deref(.)`.
Taking the value **out** of a cell is not a deref: it is the destructuring consume `let Box(value: v) = move b;` [PROV-6], the one statement that both binds the referent and releases the cell, and the one compiler-owned nominal that statement admits.
A bare borrow holder is not rebound by `set`; SET-1 requires explicit `deref(holder)` to select its referent, and only a live usable `&uniq` holder can make that referent writable [OWN-5].
There is no implicit read-through-borrow [TYPE-4, META-2].

[SET-1] Copy-place assignment.
For `set p = e;`, target evaluation first resolves and evaluates the complete `p` without reading or consuming the value stored there.
A nested place is evaluated from its base outward; at each subscript, the base place is evaluated before its offset atom, and the subscript's [OP-4] discharge obligation is judged at that target place exactly as in read position, so accepted target evaluation executes no runtime check and cannot trap.
Field suffixes introduce no runtime evaluation.

The target's final selected type is T.
The target is writable exactly when it is rooted in a live own-mode value binding whose storage is frame-resident, box-owned, arena-owned, or buffer-owned [STOR-1], or reaches a referent through an explicit `deref` of a live usable `&uniq` holder.
Fields and indices inherit the writability of their selected base except that a writable target path may traverse a view value exactly when that view's loan strength on its origin set is exclusive [VIEW-1].
A `Slice<'r, U>` is an alias-bearing shared view created by `slice_of`, and borrowing its descriptor uniquely does not grant unique access to the viewed storage, so a `Slice`-rooted target is not writable whether the descriptor binding is own-mode or is reached through another holder.
A `MutSlice<'r, U>` is the exclusive view `mut_slice_of` creates [VIEW-2], and its own loan already excludes every other access to the storage it reaches [OWN-5], so a `MutSlice`-rooted element target is writable: `set view[i] = e;` writes one element of the viewed storage, and it is the one target path this rule admits through a view.
The write is attributed to the viewed backing state and never to the descriptor [EFF-2], and it is one element write of that state under [ENT-5].
A named const is never writable [CONST-2].
A `for_stmt` binder is compiler-updated state and is never source-writable; a target rooted there is a SET-1 rejection at the complete target `place`.
A shared-borrow referent, suspended `&uniq` holder, or place conflicting with another live loan is not writable [OWN-5].
A bare borrow holder selects the holder rather than its referent and is not writable; when the holder is live usable `&uniq`, the mechanical fix is `deref(.)`.
A dead root is never writable and is not revived [OWN-1].
These specific rules own their stated violations; every other failure of this closed writability relation cites SET-1 at the complete target `place` child of the `set_stmt`, carrying the resolved root class and the required writable classes.

T is copy under [OWN-1], or is affine and admitted by [LIV-2]'s first condition at this statement's commit.
Setting a live affine place whose previous value the right-hand side does not read out remains a hard error under [STOR-1]; outside [LIV-2]'s read-out and reinitialization, `set` does not mean take, replace, or implicit destruction.
The right-hand side is then checked under [TYPE-5] and evaluated under its ordinary expression, ownership, effect, and partial-operation domain rules.
The checker analyzes the normal continuation of `e` and re-establishes there that the resolved target remains writable under the resulting loan state and that [LIV-2]'s commit conditions hold, the target root being live unless [LIV-2] reinitializes it.
If the right-hand side moved a strict prefix of the target place, the commit is a later write of a dead root under OWN-1; a `move` of the target place itself, or of a place reached through it, is [LIV-2]'s read-out and kills no root.
If it created or changed a loan that conflicts with the commit, OWN-5 rejects the commit.
This is a static acceptance check: at runtime every target component is evaluated exactly once before `e`, and lowering carries the resulting target address and offset values across `e` rather than evaluating source again.
No root-liveness or writability fact from before the right-hand side bypasses the post-state check.

A `set` target list commits the ordinals of one right-hand side [GRAM-4, CALL-4, LIV-2].
Every target is formed and judged by this rule's target judgment in written order, then the right-hand side — one `call` with that many results, or a value list of that many expressions — is evaluated, then every target is committed at one commit under [LIV-2]; target i receives ordinal i.
Two targets that overlap would make one statement's two commits order-dependent; [LIV-2]'s second condition is the disjointness judgment that decides them, so two distinct fields of one root are admitted and two subscripts of one run are not.

On successful revalidation, assignment performs exactly one write of the resulting value into `p`.
The previous value ceases to occupy `p` and requires no drop, release, finalizer, or cleanup edge [STOR-3]: a copy value needs none, and an affine value left through [LIV-2]'s read-out or was already gone.
The new value occupies the same place and the target root is live after the commit.
The store occurs only after right-hand-side evaluation completes; until that commit point the target retains its previous value.
The checked program retains the exact target path, each required target check, the right-hand-side value, the post-right-hand-side liveness and writability judgments, and the single store before lowering [DIAG-2].

[SET-2] Affine-place replacement.
`let x = replace p = e;` atomically exchanges the affine value stored at a writable place with a same-typed replacement, binding the previous value as the fresh `own` binding x [TYPE-5].
Target formation, evaluation order, subscript discharge in target position, the closed writability relation, the loan-state judgment, and the post-right-hand-side revalidation are exactly [SET-1]'s, including its specific rule attributions; every other failure of the writability relation cites SET-2 at the complete target `place`.
The target's final selected type T must be affine under [OWN-1] and region-free under [STOR-5]'s relation.
A copy-typed target is a hard error citing SET-2 at the complete target `place`, carrying T and the restructuring `use set for a copy place; read the previous value bare`.
That judgment is [FORM-1]'s one-spelling-per-class judgment and is made once per written body: at a concrete instance of a generic template it is not re-made, and a `replace` of a value whose type parameter was bounded `affine` or `linear` denotes the same exchange there [FN-2].
A region-bearing target type — `Slice<'r, U>` or `arena<'r, U>` at any depth of T — is a hard error citing SET-2 at the complete target `place`, carrying T and the restructuring `a slice's static origin set and an arena's confinement are fixed at initialization; bind a new slice or arena under a new let`; region-bearing types cannot occur in stored content [STOR-5], so this judgment bites only a direct binding or dereference target.
The right-hand side must produce exactly `own T` under the [TYPE-5] judgment stated there.
On successful revalidation, the commit performs one read of the previous value into x's storage and one write of the replacement value into resolved(p), with no writer-observable program point between them: at every program point the place holds exactly one valid owner, and no temporary uninitialized hole, vacancy state, or move-from-target residue exists.
The commit is not a consuming use of the target root under [OWN-1]: the root binding remains live, no partial-move death occurs, and the moved-out value's sole owner is x, an ordinary `own T` binding thereafter with the ordinary [OWN-1] and [STOR-3] lifecycle.
The previous value is transferred into x and the replacement into the target without changing the target's storage address or loan relationships. Subsequent effects name each destination's ordinary resolved storage [EFF-2]; a copy instantiation of an admitted affine generic performs the same exchange [FN-2].
Through a live usable `&uniq` holder the commit is the sole exception to [OWN-5]'s prohibition on moving content reached through a borrow: the exchange leaves the far-side owner owning exactly one valid T in that place at every program point, and exclusivity already excludes every other observer for the statement's duration.
A commit through a shared holder is never admitted, and a suspended holder is not usable [OWN-5].
Under [EFF-2]'s attribution the commit is one read and one write of the target's ultimate storage origin.
A successful commit derives no drop, release, finalizer, or cleanup edge [STOR-3]: nothing is destroyed, and the previous value's later release, if x is abandoned, is x's ordinary compiler-derived scope-exit action.
The exchange commits only after right-hand-side evaluation completes; before that point x is uninitialized and the target retains its previous value.
The commit is an [ENT-5] kill event exactly as stated there; it establishes no fact.
The checked program retains the exact target path, each discharged target check, the right-hand-side value, the post-right-hand-side liveness and writability judgments, the read-out, the write-in, and the binding initialization before lowering [DIAG-2].

[CONST-1] The grammar production `const` of the fence below is usable at `array<T, N>` sizes and `const` targs, and, being the `const` alternative of `targ` [GRAM-3], at every const argument of a compiler-owned container or provider nominal — a `FixedVector<T, n>` capacity and an `Arena<'s, bytes, align>`'s two constants [TYPE-2].

```wf-ebnf CONST-1
const := ("[0-9]+" | IDENT) (infix_op ("[0-9]+" | IDENT))?
```

A decimal integer literal is bare and u64 by position; an IDENT names an in-scope integer-typed const-generic parameter [GRAM-2] or a top-level integer-typed named-const item [CONST-2].
A const-expression is at most one operation over two terms, exactly the shape [GRAM-6] fixes for expressions: composition is by a named const or a forwarded const parameter, and no precedence, associativity, or parenthesization surface exists.
The tail reuses `infix_op`, and its spelling must be one of the five bare operators `+`, `-`, `*`, `/`, `%`; a mode-suffixed spelling is a hard error citing CONST-1 at the `infix_op` node, because const evaluation has no runtime overflow mode — the grammar admits and the checker restricts, META-2-clean by the `break` precedent [GIVE-1].
Constant-expressions are evaluated at monomorphization [FN-2].
An IDENT resolving to a non-integer or array-typed const is a compile-time rejection [DIAG-1].
Const evaluation is exact in the unsigned 64-bit domain under the const-eval overflow policy named `const-reject`: an operation whose mathematical result lies outside that domain, or whose divisor is zero, is a compile-time rejection citing CONST-1 at the complete `const` node.
`const-reject` is disjoint from runtime proof-required exact arithmetic: it never creates an [ENT-6] operation obligation or admits a `.defined` spelling, an accepted const-expression executes no runtime check, and a const-expression contributes no runtime effect.
Inside a generic template an unevaluated const-expression is symbolic; two symbolic const-expressions are identical exactly when their operation and ordered terms are identical, with no commutation, constant folding, or reassociation, exactly as [FN-8] fixes goal identity.
This keeps the const-generic forwarding path closed under the one operation: `const N` is usable as an `array<T, N>` size, and a derived expression such as `N * 2` is usable there and forwardable as a `const` targ, with each concrete instantiation evaluating it to one u64 value.

[CONST-2] A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value, with the `cvalue` production of the fence below.

```wf-ebnf CONST-2
cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]" | TYPEID targs? "(" (IDENT ":" cvalue ("," IDENT ":" cvalue)*)? ")"
```

`type` must be const-eligible: a primitive [TYPE-1], `array<T, N>` of const-eligible T, `FixedVector<T, n>` of const-eligible flat T [BLK-1, S34], or a source `struct` whose every field type is const-eligible; enums, `box`, `buffer`, `arena`, `Vector<'s, T>`, the two providers, the cell, and the two views [VIEW-1] are not const-eligible (a const is pure static rodata: no allocation, no region, no drop).
The `cvalue` totally defines the value: a primitive-typed const takes a FORM-5 numeric or unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T, and a struct-typed const takes the construction form `TYPEID(field: cvalue, ...)` naming its exact struct and writing every declared field in declared order [GRAM-8], each field value a cvalue of the declared field type.
The const-dependency graph is acyclic and declaration-before-use [TYPE-6]; evaluation is substitution and layout only.
A const item is never `move`d, `set`, or `&uniq`-borrowed.
It is read via subscript/`len_of` (copy-out for copy elements) or shared-borrowed `&'r p` in any region [OWN-10], so a const table may be `slice_of`-viewed and passed to a consumer.
A struct-typed const is additionally read via its field suffixes exactly as subscript reads: a copy-scalar selection copies out, and a composite selection keeps the whole-composite read rules.
A struct-typed const is laid out as one read-only static aggregate in the nominal's ordinary representation.
The `FixedVector<T, n>` const form is [S34]'s, and it is the one compiler-owned container a `const` item may write.
Its `cvalue` is `[cvalue, ..., cvalue]` with exactly `n` entries, each of type T, exactly as an `array<T, N>`-typed const's is; the count is the type's own `n` and never an entry-derived length, and a list shorter or longer than `n` is a hard error citing CONST-2 at the item.
`len_of = cap_of = n` and `room_of = head_of = Z` are standing facts of such an item [MSR-1] rather than stored words, so it **lowers to element storage only** — the run of `n` slots and no descriptor — and each use materializes the descriptor it needs from those facts.
Every read this rule already admits reads it and nothing else does: a subscript, whose `i < len_of` obligation discharges from `len_of = n` [OP-4]; the four measure readers; and a shared borrow in any region [OWN-10], so a const run may be `slice_of`-viewed at the `immutable-const` origin and is never the origin of an exclusive one [VIEW-2, CONST-2].
Enum-typed consts and written generic construction arguments in const position are DEFERRED with recorded delta: a payload-enum const has no non-consuming read path (a `match` scrutinee is an own place [OWN-13]), and a tag-only-enum const additionally needs a constant-value family no current program demands.

## 5. Ownership, regions, borrows (PROVISIONAL pending formal-calculus reconciliation)

[OWN-1] Every value has exactly one owner.
Values are classified copy or affine: primitives (TYPE-1), shared borrows, `Slice<'r, T>` [VIEW-1], and tag-only enums (every variant nullary; `Bool` is the canonical case) copy on use; all other values (owned composites, `box`, `arena`, `MutSlice<'r, T>`, uniq borrows, the two runs and the two providers [TYPE-2]) are affine.
An affine place rooted in a live own-mode binding is consumed exactly once by an explicit `move p`, by use as an own-place match scrutinee under [OWN-13], by use as the direct bare affine `Result<T, E>` place operand of `propagate` under [ERR-3], by the `move place` of a destructuring consume, or by the `place` of a `dispose` statement [PROV-6].
This classification is refined by [PROV-6] and replaced by nothing: a value affine here is linear in a scope that cannot discharge its reclamation, and a consume of a proper sub-place of such a value is that rule's partial-consume rejection rather than this rule's ordinary root kill.
Every other bare `place` expression of affine type is a hard error, and `move p` on a copy value is a hard error (copy values are used bare — one spelling per meaning, FORM-1).
That spelling judgment is made once per written body: at a concrete instance of a generic template it is not re-made, and a `move` of a value whose type parameter was bounded `affine` or `linear` denotes a copy there [FN-2, PROV-6].
The bare-affine mechanical fix is position-conditional: in a function body it is write `move p`, while in a `contract_block`, where [FN-8] rejects `move` itself, it is restate the definition or clause over copy operands or non-consuming admitted reads, so the repair never instructs a spelling FN-8 forbids.
Resolving and evaluating the target of SET-1 or SET-2 does not by itself read, copy, or move the selected value or its affine owner.
After any consuming use, the whole binding rooting `p` is dead (partial moves kill the whole binding); any later use of a dead binding, and any write or `set` of a place projected, dereferenced, or subscripted from a dead root, is an error at the later use or target place.
A `move` that is a [LIV-2] read-out of a target place of its own statement is not that consuming use and kills no root.
The [SET-2] replace commit is not a consuming use: it exchanges the stored value, leaves the target root live, and initializes its fresh binding as the moved-out value's sole owner.
SET-1 and SET-2 recheck their premises after their right-hand sides under [LIV-1]; a dead binding is revived only by a [LIV-2] commit whose target is that complete binding, which reinitializes it, and by nothing else.

[OWN-2] Modes: `own` (owned), `&` (shared borrow), `&uniq` (exclusive borrow); a borrow mode carries one region, written `&'r` or `&uniq 'r` where [FORM-8] writes it and `&` or `&uniq` where [FORM-8] elides it.
The mode itself is always written.

[OWN-3] Regions are lexical.
A `region_stmt` introduces one local region, named `region 'r { ... }` or unnamed `region { ... }`, and every `loop_stmt` or `for_stmt` body introduces one further unnamed local region whose block is that body [OWN-11]; `region_params` introduce the caller-supplied regions, and each unnamed declaration position introduces one further caller-supplied region [FORM-8].
Region identifiers are unique within a function (parameters included); an unnamed region has no identifier and is distinct from every other region of that function.
A nominal's `region_params` [GRAM-2] are its own in the same sense: a region identifier is unique within the declaration that introduces it, and two nominals of one unit may each write `'s`, exactly as two functions may.
Outlives-or-equals is the total reflexive relation: `'a` outlives-or-equals `'b` iff `'a = 'b`, or `'a`'s block strictly encloses `'b`'s block, or `'a` is caller-supplied and `'b` is local.
Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject).

[OWN-4] A borrow `&'a p` / `&uniq 'a p` held by a bound holder is live exactly until the end of `'a`'s block (named-region liveness); call-scoped temporary loans instead have the endpoints specified by [OWN-6].
It may be stored into a destination of declared region `'b`, passed to a parameter of region `'b`, or returned as `rtype` region `'b`, only if `'a` outlives-or-equals `'b`.

[OWN-5] Resolved-place exclusivity.
While `&uniq 'a p` is live and its holder is not suspended [OWN-6]: no place overlapping resolved(`p`) may be read, written, moved, or borrowed, except reads/writes through that borrow's holder and except the creation of a statement-scoped child reborrow, an arm-scoped child reborrow, a candidate-position child reborrow, or a returned reborrow of that holder [OWN-6, OWN-13, OWN-14].
While a holder is suspended (a live statement-scoped child, arm-scoped child, candidate-position child, or returned reborrow of it exists), its own read/write allowance is withdrawn: no read, write, move, copy, `set` commit, or call-transfer through it is admitted until its last child ends; a `&uniq` holder suspended by candidate-position child creation does not resume because the child loan may survive in the bound call result [OWN-6].
While any `&'a p` is live: no place overlapping resolved(`p`) may be written, moved, uniq-borrowed, or committed by `set`; reads are permitted.
A SET-1 commit is one write to resolved(target), and a [SET-2] commit is one read and one write to resolved(target); each is judged against the complete loan state after right-hand-side evaluation.
A commit through a live usable `&uniq` holder is a write through that holder; a commit through a shared holder is never admitted.
Content reached through any borrow may never be moved: `move` requires a place rooted at an own-mode binding, and the [SET-2] replace commit is the sole exception, sound because the exchange leaves no program point at which the referent place lacks exactly one valid owner.
Exclusivity invariant, checked unconditionally: no two live usable `&uniq` borrows have overlapping resolved places; a suspended holder is not usable, so the only overlapping pairs — a suspended parent with its statement-scoped child, arm-scoped child, candidate-position child, bound call-result holder, or returned reborrow — are never both-usable by construction.

Every view value — `Slice<'r, T>` or `MutSlice<'r, T>` [VIEW-1] — carries a finite set of possible ultimate storage origins.
While one function body is checked, an origin is one resolved source place, the distinguished `immutable-const` origin, or a formal-slice origin naming one of that function's parameters whose direct written type is that same view type.
Each incoming parameter of direct slice type starts with the singleton containing its own formal-slice origin, whether its written mode is `own`, `&'d`, or `&uniq 'd`; that term stands for the actual slice's complete set and is substituted only at a call boundary [FN-1, EFF-2].
Borrowing the descriptor and resolving a place through the descriptor holder preserve this set rather than replacing it with the descriptor binding's place.
Each formation row creates a singleton [VIEW-2]: a named const maps to `immutable-const`, which only `slice_of` admits, and every other admitted source retains its complete resolved place, including a place reached in arena content.
Binding, moving, passing, and returning a slice preserve the complete set.

This specification defines no slice-valued control-flow join.
A value initializer whose derived delivery type [GIVE-1, TYPE-5] is a view type [VIEW-1] is a hard error citing OWN-5 at the complete `value_match` or `value_if`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `use a match or if statement whose arms or branches return the slice directly, or call helpers with direct slice results`.
Alternative direct returns are checked independently and the caller uses their common signature ceiling [FN-1].

An access through a view is judged as one access of that view's own loan strength through every resolved-place origin in its set: shared for `Slice<'r, T>` and exclusive for `MutSlice<'r, T>` [VIEW-1].
A write, move, or unique borrow of an ordinary place conflicts when that place overlaps any such origin, at either strength.
An exclusive loan refuses the unique borrow a second *exclusive* view of its range would take, and admits a second **shared** one: that second formation is the shared child reborrow of a unique loan [OWN-6] applied to a view rather than to a place.
The child carries the parent's range, and while it is live the parent may not write the elements it views: an element write through an exclusive view is a hard error citing OWN-5 at the target while a shared loan on the storage that view reaches is live, and the parent resumes where the child's own liveness ends.
A loan's extent is its holding value's own liveness: for an affine view the consume that ends it, and for a **copy** view its last use [VIEW-1], so a write, move, or second exclusive view of the storage is admitted after the last use of every shared view of it and refused before.
A read of the origin is admitted at both strengths, which is what lets a view's own element read reach the storage it views.
`immutable-const` creates no conflicting access because named const storage is permanently read-only [CONST-2].
A formal-slice origin has a writable storage path inside its callee exactly when that view's loan strength is exclusive [SET-1]; overlap with the caller's other actual arguments is checked after substitution under [OWN-12].
No traversal order or chosen runtime arm may narrow the static set.
Each runtime slice still has exactly one actual storage origin, and that origin is always a member of the static set after complete call substitution.
Under this specification's named-region liveness, moving or returning a descriptor neither shortens nor extends the shared loan established by its source.

[OWN-6] Holder, resolution, and statement-scoped child reborrow.
The holder of a borrow is the binding its `borrow_expr` initializes. A borrow not bound by `let` is a call-scoped temporary, live until the end of the enclosing statement, except for a non-escaping control header as defined below. resolved(place) rewrites a place rooted at a holder binding to the borrowed place plus the appended suffix, recursively.
All OWN-5/OWN-7 judgments use resolved places.
The scrutinee evaluation of `match_stmt` or `value_match` is a non-escaping control header exactly when its checked mode is `own` and its checked type is an enum; the condition evaluation of `if_stmt` or `value_if` has the same boundary under [GRAM-6]'s exact `own Bool` judgment. After that evaluation has completed and before entering any arm or branch, all call-scoped temporary loans created by that header end. Temporaries created before the header retain their own extent. This is a loan boundary, not a new region: bound-holder loans [OWN-4], loans held by surviving view values [OWN-5, VIEW-1], candidate-position result loans, and arm-scoped children [OWN-13] retain their own rules. Matching through a borrow does not gain this boundary. An owned enum's payload cannot store a borrow or a view [STOR-5], and a store brand in an owned payload is not a borrow of the provider [PROV-1]. No call-return boundary is otherwise implied: temporary argument loans in SET-1, SET-2, and ordered result expressions remain live through their enclosing statement. Completion of an overlapped header requires its result and argument accesses to have completed under the sequential-equivalence and retained-loan rules [PAR-1]; observing an internal completion flag alone does not end a source loan or authorize storage reuse.
A statement-scoped child reborrow is the written form `&uniq 'c` or `&'c` over `deref(h)` followed by any written suffix chain, occurring as an argument atom of a `call` expression [GRAM-9], admitted only when: the receiving call's result mode is `own` or `unit`, never a borrow — except in the receiving call's provenance-candidate position, where a borrow result is admitted; `'c` is a locally-introduced region [OWN-3], and a caller-supplied region parameter is not admitted — except in the provenance-candidate position, where `'c` is any live region that resolved(`h`)'s region outlives-or-equals, caller-supplied included; the eligible holder `h` is a function parameter or a `let`-bound borrow, never a `match` binder; and a `uniq` child has a `uniq` parent, while a `shared` child is admitted from either [OWN-5]. resolved(child) = resolved(`h`) ++ suffix.
Creating a child suspends `h` until that child's temporary loan ends [OWN-5], at the enclosing statement end or its non-escaping control-header boundary; while a holder is suspended by this statement-scoped creation, the sole operation admitted through a place overlapping resolved(`h`) is creating a further sibling child, siblings judged by OWN-7 with any overlapping pair containing a `uniq` child an error, and `h` resumes only after its last suspending child ends. The child's region is its formation and type-validity ceiling, not the temporary loan's endpoint: its block may contain statements before and after the receiving statement. Finishing one call does not retire its temporary loans before the enclosing statement's remaining operands and commits.
Creating a candidate-position child through a `&uniq` holder suspends that holder for the remainder of its life; there is no statement-end resumption, because the child's loan may survive in the bound call result.
A shared holder needs no suspension: it admits no write through itself.
Outside the provenance-candidate exception, the child holder itself is never bound, returned, `give`n, stored, or the whole call result. Borrow-free storage [STOR-5] prevents an ordinary owned aggregate result from containing that holder. This does not erase any independently surviving loan: a result view retains every underlying-storage claim required by OWN-5 and VIEW-6, and a candidate-position borrow result retains the suspension specified above. A temporary endpoint retires only the unbound argument loan, never a bound holder, a surviving view claim, or their validity obligations.

A `let` whose ordinary right-hand side is a user call with borrow-mode result is a borrow holder rooted at the callee's provenance candidate [FN-1], and every accepted callee has one or has none.
resolved(result holder) = the candidate actual's complete resolved place, even when the callee delivered a narrower suffix of it; the holder's borrow is otherwise ordinary — OWN-4 liveness in the substituted result region, OWN-5 exclusivity, OWN-6 child admission, OWN-14 returned reborrow.
Nothing here narrows FN-1: the caller still judges the call by the signature alone.
A borrow-mode call result with no candidate is rooted in named `const` storage [FN-1, CONST-2], which no accepted write or unique borrow reaches [OWN-5, OWN-7]; its holder borrows no caller place and conflicts with nothing.

Bound children, result-carrying children (reference-result provenance), `uniq`-to-`shared` downgrade, `match`-binder parents, and written grandchild chains through a bound direct reborrow are DEFERRED with recorded delta; every written reborrow form outside this argument-atom position is dispositioned by [OWN-14], and the derived match-payload binder is [OWN-13]'s arm-scoped child reborrow.

[OWN-7] Overlap: resolved `p` overlaps resolved `q` iff one is a prefix of the other.
A resolved place is its root and the ordered steps below it — field selections and subscripts alike [MSR-1] — and two places fail to overlap exactly when some step of their common prefix provably selects two different storages: two field selections of different fields, or two subscripts whose offsets are both literals with unequal values.
The relation is therefore over the complete path and not over one offset: `grid[k]` and `grid[i][j]` are decided at `k` against `i`, and two places that agree there overlap however their later steps read.
Two slice values in a fully substituted caller context overlap conservatively iff at least one pair of their resolved-place [OWN-5] origins overlaps.
`immutable-const` needs no overlap proof because no accepted write or unique borrow of const storage exists.
Formal-slice origins are substituted before caller overlap checking [FN-1, OWN-12]; they never establish that two actual sources are disjoint.

[OWN-8] Reject-when-unsure: the checker rejects any program it cannot prove conformant.
Rejection of a sound-but-unprovable program is not a defect; the diagnostic names the rule and a restructuring.

[OWN-9] Non-normative consequence for the optimizer: a live, usable `&uniq` borrow's resolved place is unaliased by any other usable access path (a suspended holder [OWN-6, OWN-13, OWN-14] is not usable; a statement-scoped child, arm-scoped child, candidate-position child, bound call-result holder, or returned reborrow and its suspended ancestor, though both live, are never mutually noalias — the guarantee is one usable mutable path per place [OWN-5]); shared borrows are read-only for their duration; owned values are unaliased except by their own live shared borrows.

[OWN-10] Borrow-storage duration: `&'a p` is legal only if `p`'s storage outlives `'a`.
For `p` rooted at an own-mode binding b: `'a` must be introduced within b's scope (never a caller-supplied region, for locals and own parameters alike).
For `p` rooted at a borrow of region `'b`: `'b` must outlive-or-equals `'a`.
For `p` rooted in `arena<'r, T>` content: `'r` must outlive-or-equals `'a`.
For `p` rooted at a named `const` item [CONST-2]: any region `'a` is legal; immutable static storage has program lifetime and outlives every region.

[OWN-11] Loops: the body of an ordinary `loop_stmt` or a counted `for_stmt` is itself a region block.
It introduces one unnamed local region [OWN-3] whose block is that body, so that region begins and ends with one iteration and every borrow it carries is dead before the next iteration starts; outer bindings are therefore written again between iterations.
Because the region is unnamed and no position can write it [FORM-8], nothing outside the body denotes it, and a `region_stmt` that is the body's only statement is not a second way to write it [FORM-8].
Inside such a body a `borrow_expr` may denote only regions introduced inside that same loop body — the body's own region, or a `region_stmt` inside it — whether it writes the name or elides it [FORM-8].
A binding declared outside that body may be moved inside it, and the per-iteration judgment is [LIV-1]'s liveness agreement read at this loop's head: a binding declared outside the body whose live-or-dead status on the backedge differs from its status on the entering edge is a hard error citing OWN-11 at the loop, naming that binding, because one iteration would then start in a state the previous one did not leave.
A body that moves such a binding and reinitializes it before the backedge [LIV-2] agrees and is admitted; a body that leaves it dead does not.
The backedge read here is the structural one [FN-1]: it carries the state the body reached whether or not the body's own fallthrough is executable, so a body that consumes such a binding and then leaves by `break` or `return` is judged on that state exactly as a body that falls through is.
A counted binder may be copied and may be shared-borrowed only into a region introduced inside its body, but it may not be moved, uniquely borrowed, or otherwise transferred to a callee as a writable place; source writes are independently forbidden by [SET-1].
These restrictions are checked for each enclosing loop, so nesting never grants an outer binding or region to an inner body: an inner body's own region is introduced inside every enclosing body, while an enclosing body's region is introduced inside none of them.

[OWN-12] Calls (OWN-CALL cluster): at a call, declared region parameters are substituted with the caller's region arguments, which must be live; argument borrows are live accesses of their resolved places for the duration of the call and are judged under OWN-5 (two `&uniq` arguments whose resolved places overlap are an error); the callee's effect paths are projected through the corresponding actual places under [EFF-2] and checked against the caller's live borrows under OWN-5. Region substitution controls loan liveness and type equality only; it never supplies effect identity.
When an argument is a statement-scoped or candidate-position child reborrow [OWN-6], its suspended ancestor holder is excluded from this effect-row overlap check, since the child, not the ancestor, holds the loan for the call; every non-ancestor live borrow is still checked.

[OWN-13] Match ownership: a non-place expression scrutinee is an owned temporary (moved into the match).
Matching a place of own mode moves it (the binding dies; binders receive `own` payloads); matching through `&'r` / `&uniq 'r` leaves the scrutinee live and binds payloads as `&'r` / `&uniq 'r` respectively.
Binder modes are derived by this rule, stated once; they are not written.
A borrow-mode payload binder is an arm-scoped child reborrow of the scrutinee place's root binding: resolved(binder-rooted place) = resolved(scrutinee place) ++ that payload's field suffix ++ any written suffix [OWN-6], sibling binders are judged by OWN-7 with any overlapping pair containing a `uniq` binder an error [OWN-6], and creating the taken arm's binders from a `uniq`-mode root suspends that root binding [OWN-5].
Binder borrows are live until the end of their derived region's block [OWN-4], so a matched-through `uniq` root does not resume within that region; each binder is usable within its arm, and a binder borrow moved onward retains its ordinary [OWN-4]/[OWN-5]/[GIVE-1] judgments inside that same window.
Binders of a shared-mode root are overlapping shared borrows admitted by [OWN-5] without suspension.
Arm-end resumption of a matched-through `uniq` root is DEFERRED with recorded delta [META-5].
A value initializer — a `let`-initializer `match` or `if` — binds its value from its arm or branch `give`s [GIVE-1]; scrutinee treatment and binder-mode derivation are unchanged, and each delivering arm or branch delivers a value of the binding's derived mode and type [GIVE-1, TYPE-5], so on the taken arm or branch an `own` result is moved exactly once (no double-move).
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

[LIV-1] Liveness is join-checked, and that is what makes every scope-exit release unconditional.
A binding's live-or-dead status is a property of a program point, not of a path: at every join of the conservative structural normal-control graph [FN-1] and at every loop head, every predecessor agrees on the live-or-dead status of every binding in scope.
A disagreement is a hard error naming the binding and the two disagreeing predecessors.
The loop-head instance is [OWN-11]'s per-iteration judgment, which owns its stated violation, reads the structural backedge stated there, and cites OWN-11 at the loop; every other join cites LIV-1 at the join and takes the predecessors that reach it.
This agreement is judged before any capability limit of a conforming checker reports an unsupported join, so a disagreeing predecessor pair is a source rejection and never a stop.
Because the status agrees at every join, whether a compiler-derived release runs on an edge leaving a scope is not runtime state: on every edge leaving a scope — a `break`, a `give`, a `propagate` error edge, and the function-return edge included — every binding of that scope that is live on that edge takes its compiler-derived release there, unconditionally, and a binding that is dead takes none [STOR-3].
Which release runs inside a live value may still be selected by that value's own discriminant, exactly as an enum's derived drop selects on its variant today.
A binding whose value is linear in that scope [PROV-6] takes no compiler-derived release on such an edge and is refused there instead, because in that scope no derived release exists to carry it.
This rule states the liveness premise [SET-1], [SET-2] and [LIV-2] recheck after a right-hand side, and the premise [OWN-11] reads at a backedge; it adds no scope-exit action and removes none.

[LIV-2] One `set` commit.
`set (p1, ..., pn) = rhs;` for `n >= 1` writes places, the parentheses omitted at `n = 1` [GRAM-4].
The right-hand side is either one `call` producing exactly `n` results [FN-1, CALL-4] or a value list of `n` expressions evaluated left to right; at `n = 1` the two coincide in the one written expression.
Each target is a `place` [GRAM-5] — a bare binding, a field selection, a `deref`, or a subscript — formed and judged by [SET-1]'s complete target judgment in written order, including its closed writability relation and its specific rule attributions.
Every target place is resolved once, before the right-hand side is evaluated, and the resolution is not re-taken at the commit; at each subscript target the base place is evaluated before its offset atom, and every target's offsets are evaluated before the commit.
Each target's previous value is read out of resolved(p) at the start of that evaluation, and that target is dead for the remainder of it: one target is read out at most once, and a later use of what its read-out consumed is [OWN-1]'s ordinary rejection at that use.
A `move` of a target place, or of a place reached through a target place, occurring in that statement's right-hand side is that target's read-out: it is not the consuming use that kills the target's root under [OWN-1], it derives no residual cleanup of the root's unselected content, and through a live usable `&uniq` holder it is admitted on exactly [SET-2]'s exchange ground [OWN-5].
A subscripted target `P[i]` is read out by a `move` of `P[j]`, or of a place reached through it, exactly when `i` and `j` are provably the same offset — two written literals of equal value — because reading one element out and reinitialising another would leave one slot holding a value that never left and the other holding none.
An offset this rule cannot decide reads out nothing, so a live affine element target keeps [STOR-1]'s rejection there, and the `move` itself is the ordinary affine-element rejection [TYPE-2] states for every subscript read that is not such a read-out.
This read-out is the [SET-1] assignment half of [BLK-1]'s element-position commit and stands on exactly [SET-2]'s ground: no writer-observable program point lies between the read-out and the commit, so no slot is ever observed empty and no second owner of a stored element is minted.
A `move` of a strict prefix of a target place is an ordinary consuming use and kills that root.
All targets are then reinitialised at one commit, in written order, each from its own ordinal's value.
No writer-observable program point lies between the read-out and the commit, so the statement exhibits no partial move, no dead root, and no uninitialized hole, and every target is live after it.

Three admission conditions are judged at the commit, and there is no fourth.
1. Every target is dead at the commit: the right-hand side read its previous value out, the target's complete binding was already dead, or the target's final selected type is copy under [OWN-1].
A live affine target whose previous value the right-hand side does not read out is a hard error citing STOR-1 at the complete target `place`, exactly as [STOR-1] states it.
Only a target that is a complete binding is reinitialised from dead, which revives that binding; a projected, dereferenced, or subscripted target whose root is dead is [OWN-1]'s dead-root rejection at the complete target `place`, because reinitialising one component of a dead root would leave the rest uninitialized.
An affine target's final selected type must be region-free under [STOR-5]'s relation, exactly as [SET-2] requires of a replacement target; a region-bearing affine target is a hard error citing LIV-2 at the complete target `place`, carrying that type and the restructuring `a slice's static origin set and an arena's confinement are fixed at initialization; bind a new slice or arena under a new let`.
2. The targets are pairwise non-overlapping resolved places [OWN-7]: a place and any place reached through it, and two subscripted places no step of whose common prefix provably selects two different storages, are refused, because the commit order would decide the result.
Each target's place is its complete path, so `grid[k]` and `grid[i][j]` are decided at `k` against `i` and never at their last offsets.
This is a hard error citing LIV-2 at the second such target `place`, carrying both target spellings.
3. The right-hand side supplies exactly `n` values, and ordinal `i`'s mode and type are exactly target `i`'s [TYPE-5].

A target that names a binding in scope keeps that binding's [ENT-2] term and is a commit, never a declaration [TYPE-6].
Under [EFF-2]'s attribution the statement exhibits one write of each target's ultimate storage origin, and the right-hand side's own row in addition, which carries the read of every target the right-hand side reads out.
The sole exception is a complete bare binding already dead on entry to this statement: its commit initializes the new owner and exhibits no write of a previous owner's state, because no previous owner remains there. This exception preserves the right-hand side's effects, the commit's kill, the binding's term, and the liveness judgment. A target read out by this same statement was live on entry and keeps the ordinary write contribution.
For a bare binding, a field selection, or a `deref` the ultimate storage origin is that place's own storage; for a subscript `P[i]` it is the descriptor storage of `P[i]` and none of `P`'s own [MSR-2].
Each commit is an [ENT-5] kill event over that storage exactly as [SET-1]'s single commit is, and one statement's commits apply on one edge, after the right-hand side's own events and before any relation the right-hand side publishes into its targets [ENT-3.S12, CALL-6].
A commit derives no drop, release, finalizer, or cleanup edge [STOR-3]: a copy target's previous value needs none, and an affine target's previous value has already left through the read-out or was already gone.
A target identifier that resolves to no binding in scope declares one, exactly as a `let` binder does: it is a fresh declaration of the enclosing block owned by that target's own `pbase` [TYPE-6], visible after the complete `set` statement and not inside it, and its mode and type are `own` and its own ordinal's type [TYPE-5].
Such a target names no existing place, so it reads no previous value out, overlaps no other target, and is the one binding this commit initializes rather than writes; conditions 1 and 2 are over the targets that name existing places, and condition 3 fixes this one's type.
Only a bare identifier target declares: a projected, dereferenced, or subscripted target selects one component of a value that must already exist, so an unresolved base there keeps the ordinary lookup rejection [TYPE-6] states, cited as that rule's own table cites a `pbase` IDENT.
The checked program retains each target path, each discharged target check, each ordinal's value, each read-out, the post-right-hand-side liveness and writability judgments, and the one commit before lowering [DIAG-2].

[PROV-1] A store's identity is a region, that region is a component of every type the store backs, and every elided store brand resolves by one rule.
A store is a source of storage held as a value: a general store supplied by an ordinary `Heap<'s>` parameter and each bump extent a reserving occurrence produces [BLK-2].
A store region is a region that names one store.
A region denotes a store through an ordinary provider type naming it or as the store argument of a reserving occurrence [BLK-2]. Provider kinds and their region linearity obey PROV-6. No selected function name creates an implicit region.
A region may be named by at most one reserving occurrence; a second occurrence naming a region an earlier occurrence already named is a hard error citing PROV-1 at that occurrence's store `targ`, with the restructuring `open one region per store`.
Rule [OWN-3] already makes every REGIONID unique within one function declaration, so this refusal reaches exactly the case in which two reserving occurrences of one function name one region.

Every value a store backs carries that store's region in its own type, and the type fixes what the value's release needs:

| store | provider | one run | release needs |
|---|---|---|---|
| general | `Heap<'s>` | `Vector<'s, T>` | the capability of `'s`'s provider [PROV-6] |
| bump extent | `Arena<'s, bytes, align>` | `Vector<'s, T>` | nothing; `'s`'s own reset releases it |
| none | none | `FixedVector<T, n>` | nothing; its owner's frame [STOR-1] |

`FixedVector<T, n>` has no store region because it has no store, and its capacity is the type constant `n` because a frame-resident run must have a size before layout [STOR-6]; a store-resident run's capacity is a measure fixed at its formation [MSR-1], because a growth policy must be able to change it.
Preservation of a value's store needs no clause of its own: a value's store is a component of its type, no value-forming step changes a value's type, and therefore no construction, projection, element placement or removal, payload construction, match binding, result ordinal, join, argument transfer, or return changes a value's store.
Two values have the same store exactly when their types name the same region, which [OWN-12] and [TYPE-5] already decide by exact identity; a region argument of a container, of a provider, or of a source nominal carrying `region_params` [TYPE-2] is invariant there, exactly as a region parameter of any other type is.
That invariance is read where a value is produced exactly as where one is consumed: a call's result carries the store region that call determined [FN-2], so `fn f['s: affine](store: &uniq Arena<'s, bytes, align>) -> made: own BlockPool<'s>` names one type per extent and a value taken from one store is never a value of another.
A source nominal is therefore generic over its store on the same terms every other type is: `struct BlockPool['s] { free: FixedVector<Vector<'s, u8>, 8>; }` names one store per instance, `BlockPool<'a>` and `BlockPool<'b>` are two types, and a run of one may not enter the other.

Brand resolution follows ordinary declared regions.
A store region elided at a stored position — a field, an enum variant payload, a run element, or a written type argument — denotes the enclosing nominal's sole region parameter exactly when that nominal declares one region parameter.
Every other store-region position writes its region argument explicitly under FORM-8; an elision with no such enclosing parameter is a hard FORM-8 rejection at the complete type, with the restructuring `write the store region argument`.
A loan region — a view's region [VIEW-1] — retains FORM-8's ordinary parameter and elision rules. A loan region does not become a store brand by elision.

The judgment of this rule is exactly that brand resolution at every elided store-region position, the one-store-per-region refusal above, and the type equality [OWN-12] and [TYPE-5] already perform.
The checked program retains, before lowering [DIAG-2], each value's store region and the store-to-provider map [PROV-6] resolves a release against.

[PROV-6] Linearity is the reclamation half of affine, read against the scope, and closed under ownership.
A value is linear in a scope exactly when it owns, at any depth, either a value whose release action requires a capability that scope does not hold, or a value of a nominal whose declaration carries the `linear` modifier [GRAM-2]; it is affine in that scope otherwise.
This rule refines [OWN-1]'s copy/affine classification and replaces none of it: a copy value is never linear, and a value this rule does not make linear keeps exactly the disposition [OWN-1] and [STOR-3] give it.
A type owns its fields, its enum variant payloads, its `box<T>` or `Box<'s, T>` referent, its `arena` content, and the elements of an array or run it is; a loan-bearing type owns nothing, a type being loan-bearing exactly when its complete type after substitution is or reaches `Slice<'r, T>` or `MutSlice<'r, T>` [STOR-5, VIEW-1].
A full array has the same element-type ownership closure as a run: if T is linear in a scope then `array<T, N>` is linear there, including when N is zero; a zero extent changes the executed element count, not this type-level judgment.
A written type argument is owned through the field, payload, or element position it lands in and never by the type that writes it.

A type's release action requires a capability exactly when its own reclamation is a release to a store whose provider is a value.
A scope holds that capability exactly when a binding of that store's provider type is live at that point in that function, reached directly or through a borrow; a binding of a provider type enters a function only as an ordinary parameter, so a scope whose values take a compiler-derived release of that store says so in its own parameter list and its effect row carries `writes` of that provider place [EFF-2].
In this version there are two kinds of capability-released type and they behave differently under this criterion.
`box<T>` and `buffer<T>` [STOR-1] are released to the ambient heap, which is not a value: no writable type names it [TYPE-3] and no `effect_path` can be rooted at it [EFF-1], so every scope holds that capability, no value is linear by it, and every such release is exactly the compiler-derived release [STOR-3] already runs on every leaving edge [LIV-1], resolving no provider place and carrying the empty row.
`Vector<'s, T>` and `Box<'s, T>` [S39] whose store region `'s` is a general store [PROV-1] are released to that store, whose provider is the value `Heap<'s>`, so a scope holds that capability exactly when a binding of `Heap<'s>` is live at that point, reached directly or through a borrow — an ordinary written parameter in every function.
A `Vector<'s, T>` or `Box<'s, T>` whose `'s` is a bump extent is affine in every scope, because that extent's reclamation is its region's own reset and needs no capability [BLK-2].
A binding whose value owns a general-store-branded run, in a scope holding no `Heap<'s>` binding, is therefore linear there, and the release the scope exit would have run does not exist: the rejection names that scope and the absent capability.

The `linear` modifier is one optional atom on `struct_decl` and `enum_decl` [GRAM-2] and states a logical obligation, holding in every scope.
It is admitted only on a nominal [OWN-1] classifies as affine; `linear` on a tag-only enum, which [OWN-1] makes copy, is a hard error citing PROV-6 at that `enum_decl`, with the restructuring `give a variant a payload, or put the obligation on the value the issuer hands out`.
A storage obligation is never written: a value whose release action requires a capability is linear by the criterion above exactly where that capability is absent, so marking a store-derived type is redundant and marks nothing the criterion does not already see.

A value linear in this scope leaves it by exactly two routes: moved out whole, or destructured whole.
A value affine in this scope has those two, plus `dispose p;`, plus the one compiler-derived release [STOR-3] carries on every leaving edge.
A binding whose value is linear in this scope and which is live on an edge leaving that scope is a hard error citing PROV-6 at that edge's statement, naming the binding and the `linear` declaration or the absent capability that made its value linear, and offering exactly the routes that remain.

`let N(f1: b1, ..., fk: bk) = move v;` [GRAM-4] is the destructuring consume: it consumes a value of nominal struct type `N` and binds every declared field of `N` in declaration order to a fresh IDENT.
`N` is a source `struct` or the one compiler-owned nominal that has a field to bind, the cell `Box<'s, T>` [S39], whose one field is spelled `value` and whose type is its referent: `let Box(value: v) = move b;` binds the referent and releases the cell, which is why taking a value out of a cell needs no operation of its own.
Its field names are judged exactly as [GRAM-10] judges an `arm`'s: every declared field of `N` is written exactly once as `IDENT ":" IDENT` in declared order, and a missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-10 and `N`'s declared field list.
Its binders are ordinary `let` binders of the enclosing block, fresh under [TYPE-6] exactly as [CALL-4]'s binder list's are, and not arm-scoped.
Each binder receives its field's declared type and `own` mode [TYPE-5], the statement is one consuming use of `v` [OWN-1], and no residual of `v` survives it, so the statement derives no release of the consumed value's own storage [STOR-3].
An own-place `match` [OWN-13] is the enum form of the same destructuring.

The release graph of a type `T` has as its nodes the types reachable from `T` through fields, enum variant payloads, cell referents — `box<T>`'s and `Box<'s, T>`'s alike — `arena` content, and array or run elements; a loan-bearing value contributes no node.
A type's release action is non-empty by the least fixed point of two clauses: a capability-released type is non-empty, and any type owning a non-empty type is non-empty [STOR-3].
The graph has an edge from a node to a sub-node exactly when that sub-node's release action is non-empty.
One walk performs both the compiler-derived release and `dispose`, and it visits exactly the nodes of that graph in [STOR-3]'s order — every field of a struct in declaration order, an enum's active variant's payload selected by the discriminant, a cell's referent before the cell itself, every element of an array or run in ascending logical index order — releasing at each capability-released leaf to the store its own type names and spending that store's resolved provider, and running each other non-empty leaf's ordinary release action.
A field, payload, or element whose release action is empty is never visited, and a container's elements are visited before its backing is released, so a release of a full container needs no emptiness premise.
A type whose release graph has a cycle makes that walk's depth a runtime quantity rather than a compile-time constant, and is admitted: the derived release of such a type is one release action per node type, entering itself where the graph closes, and the walk's depth is the value's own.
A cycle in a release graph can arise only where a heap is allowed — an arena-resident recursive node's release action is empty, so the walk never enters it — so a runtime-quantity release depth is a property of exactly the programs whose resource behaviour is already a runtime quantity.
Every judgment of this rule that reads the graph reads each node once, which terminates on a cyclic graph and is exactly the node set this version's release actions need.

`dispose p;` [GRAM-4] is the early release: it runs at the point it is written the same walk the scope exit would run, and it names no capability.
It is admitted only when `p`'s release graph contains at least one capability-released leaf, when this scope holds the capability of every such leaf, and when no node of that graph — `p`'s own type included — is linear by the modifier.
A `p` whose release graph contains no capability-released leaf is a hard error citing PROV-6 at the complete `place`, with the restructuring `this value's release action reclaims no capability; let the scope exit run it`.
A `p` one of whose release-graph nodes carries the `linear` modifier is a hard error citing PROV-6 at the complete `place`, naming that node, with the restructuring `take the value apart with let N(f: a, ...) = move v; and discharge the marked component`; the modifier can be written only on a struct or an enum, which the walk never treats as a leaf, so the condition is stated over nodes and `p`'s own type is one of them.
For each capability-released leaf, let `'s` be the store region its type names and `P('s)` the provider type of `'s`'s store; the statement resolves the innermost live binding of this function whose type is `P('s)`, reached directly or through a borrow, and writes it.
No such binding in scope, or only one reached through a shared borrow, is a hard error citing PROV-6 at the complete `place` with the missing parameter rendered.
A leaf that names the ambient heap resolves no binding and contributes no provider write, its provider not being a value; a leaf that names a general store resolves that store's live `Heap<'s>` binding and contributes `writes` of it, which is what makes an early release of a store-backed run visible in its function's declared row [EFF-2].
The statement is one consuming use of `p`'s root [OWN-1]: `p` must be rooted in a live own-mode binding of this function, so a `p` reached through a shared or an exclusive borrow receives [OWN-1]'s ordinary rejection at the complete `place`.
That root's type must not be loan-bearing and its release graph must contain no loan-bearing node; either violation is a hard error citing PROV-6 at the complete `place`, with the restructuring `a view owns nothing and has no release action of its own; release the value it views`.
The statement exhibits one write of `p`'s ultimate storage origin, the same origin [LIV-2] writes at a commit, beside the write of each resolved provider place, so [EFF-2] projects it exactly as it projects a commit's write.

A consume of a proper sub-place of a value linear in this scope is a partial consume exactly when that sub-place is not reinitialised at the same statement's commit [LIV-2].
A partial consume is a hard error citing PROV-6 at the complete consumed `place`, naming the residual, with the restructuring `destructure the whole value with let N(f: a, ...) = move v;`.
The refusal is stated over the consume, so it reaches `dispose p.f;` exactly as it reaches `let x = move p.f;`.
A [LIV-2] target list every member of which is reinitialised at that statement's one commit leaves no residual and is therefore not a partial consume; every other consume of a sub-place of such a value is.

A declared region parameter `'s` is an arena region when the declaration writes a parameter or a result whose type places a store of `'s` under a region release [STOR-4], a heap region when it writes one whose store's provider is a value of heap type, and unconstrained otherwise; a value branded by an unconstrained `'s` is treated fail-closed as capability-released.
A function that declares a region parameter `'s` may not let an own-mode value that owns, at any depth, a capability-released leaf branded `'s` reach a scope exit by a compiler-derived release unless `'s`'s class is arena or the declaration holds `'s`'s provider.
Its four routes are exactly the ones above — move the value out by a result, destructure it whole, dispose it, or take the compiler-derived release — the last two available exactly under that condition.
The check is at the declaration, over the body, once; a violation is a hard error citing PROV-6 at the `fn_decl`, naming the region, the binding, and both repairs.
Each view type [VIEW-1] is loan-bearing and owns nothing, and `arena<'r, T>`'s storage is released with its region [STOR-4], so neither contributes a region-branded capability-released leaf; `Vector<'s, T>` does, so this obligation refuses exactly a declaration that receives a `Vector<'s, T>` by value over an unconstrained `'s`, lets it reach a scope exit, and holds neither `'s`'s provider nor an `'s: affine` bound.

A region parameter written `'s: affine` or `'s: linear`, and a type parameter written `T: copy`, `T: affine` or `T: linear` [GRAM-2], is bounded rather than unconstrained: the bound names the linearity class the declaration is written for, so the obligation above is checked once against the bound instead of fail-closed.
The three classes form the strict chain `copy < affine < linear`, ordered by what a body may do with a value of the class: under `copy` the body may duplicate the value, use it bare, and drop it; under `affine` it may `move` it at most once and may drop it; under `linear` it must consume it exactly once and may never drop it.
`copy` names [OWN-1]'s copy class; `affine` names the class whose values need no capability at reclamation and are not marked by the modifier; `linear` names the class whose values need one or carry the modifier.
A type parameter's bound is always written and never inferred, and every type parameter of a function or a nominal carries exactly one [FN-2, GRAM-2]; a region parameter's is optional, and an unbounded region parameter is the unconstrained case above.
A region parameter's bound is `affine` or `linear` and never `copy`: the bound names the class of the *store* the region identifies [PROV-1], and no store is reclaimed by duplication, so `'s: copy` is a hard error citing PROV-6 at that `region_param`, with the restructuring `a region parameter's bound names its store: write 'affine' for a bump extent and 'linear' for a general store, or leave it unbounded`.
Satisfaction is that chain read left to right: an argument of class C instantiates a bound B exactly when C <= B, so `copy` accepts copy arguments only, `affine` accepts copy and affine, and `linear` accepts every class.
An instantiation whose argument's class does not satisfy the written bound is a hard error citing PROV-6 at that instantiation's `call`, naming the parameter, the bound, and the argument.
A type parameter bounded `linear` is linear at the one symbolic instance its body is checked at [FN-2], so a body that lets such a parameter's value reach a scope exit receives this rule's own not-consumed rejection there, naming the written bound in place of a `linear` declaration.
The same check applies on the region axis: a region argument's class is `affine` when the store it names is a bump extent, whose reclamation is its own region reset, and `linear` when it names a general store, whose reclamation spends a provider capability [PROV-1]; an instantiation whose region argument's class is not the written bound is the same hard error at that instantiation's `call`, naming the region parameter, the bound, and the argument.
A region argument that names no store — a loan region, or a region introduced by a `region_stmt` that no reserving occurrence names — has no store class and satisfies neither bound; an instantiation supplying one to a bounded region parameter is the same error.
The bound is a linearity class: it supplies no function-kind argument, selects no behavior, and creates no bound-satisfaction judgment other than this one [FN-2, FN-3].

The checked program retains, before lowering [DIAG-2], each scope's linear set, each `dispose` statement's walk and the write it exhibits, each destructuring consume's binder list, and each declaration bound that was checked.

[BLK-0] There is exactly one compiler-owned kernel declaration domain, and it is generic.
The container and store operations are that domain: a third admitted declaration source alongside source declarations and the prelude [PRE-1], admitted by every compilation unit.
It contributes one declaration class and no other: each operation takes the function class and is an entry of the lexical IDENT domain [TYPE-6], visible throughout the closed unit, colliding with a source declaration of the same spelling under [DIAG-1] exactly as a prelude function does.
Its owner-local type, const, region, and value parameter records are visible only within their owning operation and never enter source lookup.
It contributes no nominal type: the container and provider nominals are [TYPE-2] types, not entries of this domain.

Each operation is one complete signature record: its type, const, and region parameters in the order [GRAM-2] writes them; its named value parameters in declared order [GRAM-11]; one declared effect row [EFF-1]; one declared result mode and type, or one ordered result list [FN-1]; one declared requirement list; and one declared relation list.
The first declared value parameter is the value the operation transforms and returns; an operation that transforms nothing names its provider first; and one that neither transforms nor provides names the value it observes first.
The inventory is [BLK-2]'s and [BLK-3]'s rows, and this rule is that the domain exists, that every row satisfies the sentences below, and that every row's first-parameter ordering is the one just stated.

Written arguments are decided per argument, not per callee.
A call to a kernel-domain operation writes each region argument exactly where [FORM-8] writes it — that is, exactly when no operand of that row determines it — and writes each type or const argument exactly when no operand of that row supplies it, which is [TYPE-5]'s retained-argument sentence applied to a fourth callee class.
So `heap_vector::<u8>(store: heap, count: n)` writes `T` and elides `'s`, which its `store` operand supplies; `heap_box(store: heap, value: e)` writes nothing at all, its `value` operand supplying `T` and its `store` operand `'s` [S39]; `arena_vector::<u8>(store: arena, count: n)` writes `T` and elides `'s`, `bytes`, and `align`, all three of which its `store` operand supplies; and `place_back(vector: v, value: e)` writes nothing.
The conversions `array_from_fixed(vector: v)` and `fixed_from_array(values: a)` [BLK-3] likewise write nothing: their operand types supply both T and n.
A written argument this criterion does not require, or a missing one it does, is a hard error citing BLK-0 at the `call`, naming the operation.
A user `fn` generic remains the other class and always writes its type and const arguments [FN-2].

A kernel-domain call writes its value arguments as a `fieldinit_list` [GRAM-5] whose IDENTs equal the declared parameter names in declared order, under exactly the discipline [GRAM-11] applies to a user function; positional operands are not admitted.
Every declared value-parameter and result spelling of the inventory therefore satisfies [FORM-3]'s IDENT class: a spelling a fixed grammar atom already produces could not be written at a call and would make its row unreachable, so the inventory names the one provider operand of every acquiring row `store` and not after the store's own nominal.
A kernel-domain operation cannot be a function-kind argument or the right callee of an [FN-3] `fn_bind`; FN-4 binds only an explicitly instantiated ordinary function or a forwarded function parameter.

Every row is complete over every measure it writes, on every exit.
A row whose effect row carries `writes(P)` for a measured place `P` publishes, for each measure of `P` [MSR-1], its exact new value where that measure's cell is exact and a two-sided bound where that cell is bounded, including the measures it did not change, and on every declared exit including a refusal.
Completeness is read with [MSR-2]'s standing identity: `room_of` is the complement `len_of(P) + room_of(P) = cap_of(P)` already determines with empty support, so a row that publishes `len_of` and `cap_of` of a place has published its `room_of` and needs no third clause for it.
A row may publish `room_of` explicitly anyway, and the boundary rows [BLK-3] do; the formation rows [BLK-2] do not, and both classes are complete under the same sentence.
A row's declared relations are published exactly as a source [FN-9] relation set is: instantiated at the call and established on its continuation by [CALL-6], with each operand substituted at the denotation [MSR-3]'s table gives its parameter's mode, and with each routed relation restricted to its own arm [CALL-4].
A kernel-domain record's requirement and relation lists are normative record notation and are not a source `contract_block`, so [FN-9]'s admission conditions — which datum shapes a clause operand may take, and which variant a route may name — quantify over a source-declared block and reach no record of this domain: a row's relation may name a measure of its own result, may be routed to any variant of the enum that result is, and may name a measure of a `&uniq` state parameter, denoting that parameter's post-state there.
What a caller may derive from an established relation is unchanged by this: the relation is instantiated, established, and killed exactly as [CALL-6] states for every other declared relation in the language.
The form `<measure>(deref(entry(parameter)))` denotes that call's call datum for the same place [MSR-3], with the same `entry` former and explicit dereference as a source contract, and the post-state occurrence of the same measure is the live term after the call's own kills; the two are two terms wherever both occur, even though the parameter, the place, and the measure are the same one.
A row's declared effect row is a callee effect [ENT-5] like any other: the place a row's `writes` names is written by the call, so every fact whose support that place reaches dies at the call boundary and what a caller holds about that place afterwards is exactly what the row published.

A row's published set is subject to [CALL-6]'s consistency judgment exactly as a source declaration's is.
The relations one row carries on one declared exit, together with that row's own requirements, are not contradictory.
The half of that set a caller reaches on *every* exit — the row's unrouted relations — additionally never makes the caller's fact state contradictory where it was not already, every requirement of the row having been discharged before any relation of it is established [MSR-4]: those relations hold wherever the call's continuation is reached at all, so a state that turns contradictory across them turned so on the row's own relations.
A contradictory published set is not one wrong fact: at a contradictory point every relation and both signs of every goal are derivable [ENT-4], so the caller discharges every obligation it submits after that call, the subscript bounds and the integer domains among them.
A caller state that turns contradictory across a row's *routed* relations is a different thing and is admitted: a routed relation is available only on the arm its route names [CALL-6], so a contradiction there is the ordinary [ENT-3] statement that this arm is not reached, exactly as a written guard the caller can refute makes its own arm underivable.
An acquisition asked for more bytes than the store can hold publishes `len_of(deref(store)) = len_of(deref(entry(store))) + advance<T>(count)` on its `Some` arm against a `cap_of(deref(store))` that cannot hold it, and the arm it makes underivable is the arm that never runs.
The exits of one call partition its outcomes, so at most one of them is refuted this way and the caller reaches a consistent state on the arm it takes.
Where a row names one measure of one formal both through `entry(parameter)` and in its post-state, the two are two terms at every instantiation [MSR-3]; a caller that read one term for both would give the row's own relation the shape `t = t + advance<T>(count)`, which is the bound pair `advance<T>(count) <= 0` and `advance<T>(count) >= 0` and, at any nonzero take, a contradiction the row introduces into every caller on every exit.
Because a row's set is fixed by this document rather than by a program, a row that fails this judgment is a defect in this document or in its implementation and is never a source rejection.

A row's operands are [ENT-2] terms, constants, and exactly the compiler-owned formers this rule defines.
`size_ceiling(T)` and `align_ceiling(T)` are the layout ceilings [OP-9] fixes, and are compile-time constants of one concrete instance.
`advance<T>(count)` is one [ENT-2] term of fragment type `u64`, equal to `round_up(stride_ceiling(T) * count, align)` where `align` is the store's own type constant [OP-9]; its support is the support of `count`, so it is a symbolic constant when `count` is closed and an opaque term otherwise, and a relation over it is an ordinary difference bound between two terms.
The quantity is the stride and not the size because a run's slots are stride-spaced [BLK-1]: `count` of them occupy `stride_ceiling(T) * count` bytes, and a take of `size_ceiling(T) * count` would hand out a run whose last slots lie outside what the store gave it.
A row's declared requirement is one obligation its caller discharges [MSR-4], so a requirement naming `advance<T>(count)` at an open count states a bound over an opaque term the caller has no source spelling for; a caller in that position discharges nothing and the acquisition is the refusing row's.
`fits::<T>(count)` is not a term: it is the record-notation spelling of [OP-9]'s allocation-fit obligation over `(T, count)`, discharged by [OP-9]'s own judgment under [MSR-4].
Every acquiring row carries that obligation and `fixed_vector` carries none, because its count is a type constant [STOR-6] governs.

The readers are not in this domain.
`len_of`, `cap_of`, `room_of`, and `head_of` are four [OP-1] table operations over a bare non-consuming place operand, returning `own u64`, and `pure`; a caller reading a measure of a borrowed place exhibits `reads` of it by [EFF-2]'s ordinary attribution, and a `let` binding one of them establishes the ordinary [ENT-3.S6] equality over that measure term.

The judgment of this rule is row resolution by spelling, operand types, and written arguments; the per-row requirement discharge under [MSR-4], the allocation-fit obligation included; the [GRAM-11] named-argument check; the per-argument written-argument check above; and the completeness check over each row's published relation set, which is a property of this specification's data established once for this document.
A diagnostic arising in this domain cites BLK-0 and names the operation in its payload, exactly as an [OP-1] diagnostic cites OP-1 and names its family.
Each inventory entry has one zero-based `container_declaration_ordinal` assigned by the preorder of [BLK-2]'s rows followed by [BLK-3]'s, and within one operation each type, const, and region parameter in declared order followed by each value parameter in declared order; that ordinal is the entry's identity in a diagnostic origin [DIAG-1].

[BLK-1] Two runs, one shape, one window, and what a slot may hold.
`FixedVector<T, n>` and `Vector<'s, T>` are the two runs of slots and there is no third.
`FixedVector<T, n>`'s capacity is the type constant `n` [CONST-1] and its storage is inline in its owner or its owner's stack frame; `Vector<'s, T>`'s capacity is a measure fixed at its formation [MSR-1] and its storage is one run taken from the store `'s` names [PROV-1].
Each is a run of `cap_of` slots whose initialized storage is a window: exactly the `len_of` slots beginning at `head_of` modulo `cap_of`, every other slot raw.
A run carries no per-slot tag, no occupancy bitmap, and no runtime discriminant; the window is its complete typestate and `len_of`, `cap_of`, `room_of`, and `head_of` are its complete measure row [MSR-1].
A subscript `v[i]` selects the element at logical offset `i` and carries [OP-4]'s obligation `i < len_of(v)`, stated against `len_of` and never against `cap_of` or `head_of`; the storage it selects is slot `(head_of(v) + i) mod cap_of(v)`, which is the coordinate system [MSR-1] fixes and where that rule's injectivity sentence applies.
Read position and target position select it the same way: a subscript in the target of a [SET-1] `set` or a [SET-2] `replace` is the same logical offset, carries the same [OP-4] obligation judged as that rule states, and reaches the same slot of the window.
An implementation that has proved `head_of(v)` identically zero may emit the plain `base + i * stride` form; acceptance never depends on whether it has.
A `Vector<'s, T>` of capacity one is a run of one slot and not a cell: [S39]'s `Box<'s, T>` is the store-resident single-value nominal, and the reason it exists rather than the one-slot run is measured — a run carries three descriptor words and owes `0 < len_of(v)` at every read, while a cell is never empty and carries none.
An element type `T` may be copy, affine, or linear [OWN-1, PROV-6]: an element enters or leaves a continuing run through an operation that moves a boundary [BLK-3] or through an element-position commit at a subscript of the run — one [SET-1] assignment or one [SET-2] replacement — and none leaves an initialized slot empty nor reads a raw one.
A consuming representation conversion [BLK-3] instead transfers the complete initialized set and leaves no source run alive.
A run whose element type is linear in a scope owns its elements, so a value of that run type is linear in that scope too and [PROV-6]'s release walk visits exactly its window.
Neither run inherits [TYPE-2]'s buffer element restriction: a run's element type is any nameable type [TYPE-3], a measured type and a type parameter under any of its three bounds included [PROV-6], and a measure term over `P[i]` is the ordinary [MSR-1] term over that element place.
Each of the five compiler-owned nominals [TYPE-2] contributes one nominal-type entry and one constructor entry of the same spelling [TYPE-6], exactly as a source `struct_decl` does, and the two do not collide because the grammar distinguishes a `type` role from a constructor `call` role.
The constructor entry exists to be refused: no constructor `call` produces a run, a provider, or a store, so a constructor `call` [GRAM-8] admitting one of those five constructor entries is a hard error citing BLK-1 at the complete constructor `call`, with the restructuring `form the run with a formation operation, or receive the provider as a parameter`.

[BLK-2] Formation, one row per placement and one per store.
Six formation rows and one reservation row, in this preorder, each one complete signature record in the [GRAM-2] `fn_sig` shape extended by this domain's requirement and relation lists [BLK-0]:

```
fn fixed_vector<T, const n: u64>() -> result: own FixedVector<T, n> pure
  ensures len_of(result) == 0_u64;
  ensures cap_of(result) == n;
  ensures room_of(result) == n;
  ensures head_of(result) == 0_u64;

fn arena_vector<T, const bytes: u64, const align: u64>['s](store: &uniq Arena<'s, bytes, align>, count: own u64)
    -> made: own Option<Vector<'s, T>> reads(store), writes(store), allocates(store)
  requires align >= align_ceiling(T);
  requires fits::<T>(count);
  ensures when made is Some(value: r): len_of(r) == 0_u64;
  ensures when made is Some(value: r): cap_of(r) == count;
  ensures when made is Some(value: r): room_of(r) == count;
  ensures when made is Some(value: r): head_of(r) == 0_u64;
  ensures when made is Some(value: r): len_of(deref(store)) == len_of(deref(entry(store))) + advance<T>(count);
  ensures when made is None(): len_of(deref(store)) == len_of(deref(entry(store)));
  ensures when made is None(): room_of(deref(store)) < advance<T>(count);
  ensures cap_of(deref(store)) == cap_of(deref(entry(store)));

fn arena_vector_proved<T, const bytes: u64, const align: u64>['s](store: &uniq Arena<'s, bytes, align>, count: own u64)
    -> result: own Vector<'s, T> reads(store), writes(store), allocates(store)
  requires align >= align_ceiling(T);
  requires fits::<T>(count);
  requires room_of(deref(store)) >= advance<T>(count);
  ensures len_of(result) == 0_u64;
  ensures cap_of(result) == count;
  ensures room_of(result) == count;
  ensures head_of(result) == 0_u64;
  ensures len_of(deref(store)) == len_of(deref(entry(store))) + advance<T>(count);
  ensures cap_of(deref(store)) == cap_of(deref(entry(store)));

fn heap_vector<T>['s](store: &uniq Heap<'s>, count: own u64)
    -> made: own Option<Vector<'s, T>> reads(store), writes(store), allocates(store)
  requires fits::<T>(count);
  ensures when made is Some(value: r): len_of(r) == 0_u64;
  ensures when made is Some(value: r): cap_of(r) == count;
  ensures when made is Some(value: r): room_of(r) == count;
  ensures when made is Some(value: r): head_of(r) == 0_u64;

fn arena_box<T, const bytes: u64, const align: u64>['s](store: &uniq Arena<'s, bytes, align>, value: own T)
    -> made: own Result<Box<'s, T>, T> reads(store), writes(store), allocates(store)
  requires align >= align_ceiling(T);
  ensures when made is Ok(value: b): len_of(deref(store)) == len_of(deref(entry(store))) + advance<T>(1_u64);
  ensures when made is Err(error: back): len_of(deref(store)) == len_of(deref(entry(store)));
  ensures when made is Err(error: back): room_of(deref(store)) < advance<T>(1_u64);
  ensures cap_of(deref(store)) == cap_of(deref(entry(store)));

fn heap_box<T>['s](store: &uniq Heap<'s>, value: own T)
    -> made: own Result<Box<'s, T>, T> reads(store), writes(store), allocates(store)

fn arena_frame<const bytes: u64, const align: u64>['s]() -> result: own Arena<'s, bytes, align> pure
  ensures len_of(result) == 0_u64;
  ensures cap_of(result) == bytes;
  ensures room_of(result) == bytes;
```

`fixed_vector` needs no store and is `pure`; each of the five acquiring rows takes its store's provider as a `&uniq` parameter and publishes that store's post-state measures, of which `Heap<'s>` has none.
The two cell acquisition rows [S39] consume an element that may be affine, so their refusal cannot be an `Option`: a refusal that dropped `value` would destroy what the caller handed over [L3], and each therefore hands back `Result<Box<'s, T>, T>`, whose `Err` arm carries the value itself.
Neither cell row spells a generic argument at the call: `T` is supplied by the `value` operand and `'s` — and, for the arena row, `bytes` and `align` — by the `store` operand, so [BLK-0]'s per-argument criterion leaves the call `heap_box(store: s, value: e)`.
`advance<T>(1_u64)` in the arena row is the record notation's own quantity for one cell: one stride rounded up to the store's alignment constant, exactly as a take of one slot is.
Each arena row additionally requires `align >= align_ceiling(T)` as a compile-time comparison of two constants, which is what makes the bump cursor a multiple of `align` at every program point, the padding at a take zero, and `len_of(arena)` exact [MSR-1].
A fallible run acquisition returns an `Option`: its count is copy and its provider is borrowed, so a refusal has no consumed element to hand back. A proved run acquisition has no refusal result. The cell acquisition rows instead return their consumed element on refusal as stated above.
The general store has no proved form, because no honest compile-time domain predicate exists for it; the arena has one, whose `room_of` requirement [MSR-4] discharges and whose failure is therefore a static rejection with no runtime fallback.
`Heap<'s>` has no formation row: a general-store provider is an ordinary argument supplied with its explicit region [PROG-3].

`arena_frame::<bytes, align, 's>()` reserves one bump extent per activation of the region block naming `'s`, laid out in that activation's own frame.
Its written `'s` must be a region introduced by an enclosing `region_stmt` of the reserving function; a caller-supplied region parameter is not admitted, and the occurrence must be a statement of that region block and of no loop inside it.
Either violation is a hard error citing BLK-2 at the occurrence's store `targ`, with the restructuring `move the region block inside the loop, so the store is reserved and reset per iteration`.
Rule [PROV-1] admits at most one reserving occurrence per region.
The reservation is where the extent's initial state is established, at every activation of that block: the reserving occurrence sets the bump cursor to zero, so an extent whose region block is entered again is reserved again in the same frame storage and starts empty.
An extent therefore carries no release action of its own and contributes no row to [STOR-3]'s table: its storage is the reserving activation's frame, which that activation's own return reclaims, and nothing of it is observable outside its region block, so no edge leaving that block has work to do.
DEFERRED: a second reservation row `arena_extent`, which produces its own resource-envelope item instead of a frame contribution, together with the refusal of an occurrence more than one activation of whose region block can be live at one program point; its delta is numbered rules +0, grammar productions +0, and records +1.
It is deferred because the refusal quantifies over call-graph components and execution contexts that this version's resource judgment does not state, and a reservation whose per-activation identity is unchecked would publish `len_of(result) == 0_u64` falsely.

[BLK-3] Boundary operations and complete-value conversions.
`V` is a compiler-owned run type parameter of this domain whose admitted arguments are exactly `FixedVector<T, n>` and `Vector<'s, T>` and whose element type is that run's own; it is supplied by the `vector` operand and never written, and no source declaration can write such a parameter.
Four boundary rows followed by two representation-conversion rows, in this preorder, continuing [BLK-2]'s inventory:

```
fn place_back(vector: &uniq V, value: own T) -> result: own unit reads(vector), writes(vector)
  requires room_of(deref(vector)) > 0_u64;
  ensures len_of(deref(vector)) == len_of(deref(entry(vector))) + 1_u64;
  ensures room_of(deref(vector)) + 1_u64 == room_of(deref(entry(vector)));
  ensures cap_of(deref(vector)) == cap_of(deref(entry(vector)));
  ensures head_of(deref(vector)) == head_of(deref(entry(vector)));

fn place_front(vector: &uniq V, value: own T) -> result: own unit reads(vector), writes(vector)
  requires room_of(deref(vector)) > 0_u64;
  ensures len_of(deref(vector)) == len_of(deref(entry(vector))) + 1_u64;
  ensures room_of(deref(vector)) + 1_u64 == room_of(deref(entry(vector)));
  ensures cap_of(deref(vector)) == cap_of(deref(entry(vector)));
  ensures head_of(deref(vector)) >= 0_u64;
  ensures head_of(deref(vector)) <= cap_of(deref(vector));

fn take_back(vector: &uniq V) -> value: own T reads(vector), writes(vector)
  requires len_of(deref(vector)) > 0_u64;
  ensures len_of(deref(vector)) + 1_u64 == len_of(deref(entry(vector)));
  ensures room_of(deref(vector)) == room_of(deref(entry(vector))) + 1_u64;
  ensures cap_of(deref(vector)) == cap_of(deref(entry(vector)));
  ensures head_of(deref(vector)) == head_of(deref(entry(vector)));

fn take_front(vector: &uniq V) -> value: own T reads(vector), writes(vector)
  requires len_of(deref(vector)) > 0_u64;
  ensures len_of(deref(vector)) + 1_u64 == len_of(deref(entry(vector)));
  ensures room_of(deref(vector)) == room_of(deref(entry(vector))) + 1_u64;
  ensures cap_of(deref(vector)) == cap_of(deref(entry(vector)));
  ensures head_of(deref(vector)) >= 0_u64;
  ensures head_of(deref(vector)) <= cap_of(deref(vector));

fn array_from_fixed<T, const n: u64>(vector: own FixedVector<T, n>)
    -> result: own array<T, n> reads(vector)
  requires len_of(vector) == n;
  ensures len_of(result) == n;
  ensures cap_of(result) == n;
  ensures room_of(result) == 0_u64;
  ensures head_of(result) == 0_u64;

fn fixed_from_array<T, const n: u64>(values: own array<T, n>)
    -> result: own FixedVector<T, n> pure
  ensures len_of(result) == n;
  ensures cap_of(result) == n;
  ensures room_of(result) == 0_u64;
  ensures head_of(result) == 0_u64;
```

`place_back` and `take_back` move the back boundary and leave `head_of` where it was; `place_front` and `take_front` move the front boundary, and `head_of` is the one measure whose cell is bounded [MSR-1], so those two rows publish it two-sidedly and no row re-establishes it exactly.
Each boundary row mutates its exclusive run referent in place. A placement returns unit and a take returns only the removed element. Measures through `entry(vector)` denote the call datum; measures through `vector` denote the referent after the call. These are distinct terms [MSR-3]. A boundary operation neither transfers the run owner nor changes its backing address.
Element access is the ordinary surface over the initialized window and needs no row: `v[i]` reads, `set v[i] = e;` writes a copy element [LIV-2], and `let old = replace v[i] = e;` exchanges an affine one [SET-2].
There is no swap, exchange, rebase, growth, clear, truncate, removal from the middle, or vacant construction anywhere in this domain: a swap of two whole non-overlapping places is `set (p, q) = move q, move p;` [LIV-2], a swap of two elements of one run is the same one commit over its two subscripts, `set (v[i], v[j]) = move v[j], move v[i];`, whose offsets [LIV-2]'s second condition requires to be provably distinct and whose read-outs that rule's own sentence admits, and each remaining item is an ordinary source function over these rows.
No boundary row is total at a capacity or an emptiness boundary, because an overwriting or an empty-take form would have to publish a displacement or a refusal this domain declares no value for.

`array_from_fixed` consumes the complete initialized window and transfers element i in its logical order to array index i, for every i less than n.
Its sole requirement is fullness; `head_of(vector)` need not be zero. A wrapped window is transferred in logical order just as an unwrapped one is, and the source has no live residual.
`fixed_from_array` consumes the full array and transfers its element i to slot i of a full run whose head is zero.
Both conversions preserve the exact element type and each element's ownership, store identity, and eventual release action [PROV-1, PROV-6]; neither allocates backing, releases an element, nor duplicates an affine owner.
They may move inline payloads between their representations; neither promises address preservation or a zero-copy transfer, and the ordinary move and loan judgments apply [OWN-1, OWN-5].
`array_from_fixed` reads the source window's coordinates; `fixed_from_array` repacks a complete owner without inspecting or changing its elements [EFF-2].
At n equal to zero both conversions transfer no elements and perform no element access or release; a zero-byte element representation does not remove its logical element occurrences or their ownership obligations [STOR-6, PROV-6].

[BLK-4] Confinement and the one position closure.
A type is confined when its complete type after substitution names a region, and the confinement of a value is the set of regions its complete type names.
A confined value may be moved, returned, or bound to a destination that every member of that set outlives or equals [OWN-3]; the quantifier is the whole of it, because [OWN-3] makes two caller-supplied regions incomparable and fail-closed is the answer there.

A confined value may occupy any position whose owning value's own complete type names the same region, so the position is itself confined and [STOR-4] governs it.
That is what admits a store-branded run into a field, an enum variant payload, and a run element: the store's identity travels in the type [PROV-1], and nothing about the value outlives, hides, or strands anything.
A loan-bearing type [VIEW-1] may occupy no position from which a value could outlive or hide its origin set — no field, no enum variant payload, no run element, no written generic type argument, and no result outside [VIEW-6]'s ceiling — and a provider type may occupy none of the same positions, because a moved provider strands its own store [STOR-5].
Rule [STOR-5] states that closure over the stored positions and [FN-2] over the written type argument; this rule is where the two are one judgment.

In an ordinary function's parameter list, a `&uniq` referent may reach a run or a generic type parameter through any number of fields, enum variant payloads, array or run elements, and written type arguments.
Caller facts survive exactly when the callee's exact effect row projected onto resolved places does not write their support [CALL-6, ENT-5]; verified two-state relations establish after those kills [FN-9, MSR-3].
Whole replacement through an opaque type parameter is a write to that resolved referent, and the same storage-overlap kill removes facts about any element it contains.
The stored-position, confinement and loan-bearing judgments still apply, as do the view replacement restriction [VIEW-4] and provider identity [PROV-1].

The judgment of this rule is the destination check over a confined value's region set; the stored-position half is STOR-5's own judgment and is not repeated here.
The checked program retains, before lowering [DIAG-2], each value's confinement set.

[VIEW-1] Two views, one shape, two loan strengths.
A **view** is a value that reaches storage it does not own [OWN-5]: `Slice<'r, T>` holds a **shared** loan on the range it reaches and reads it, and `MutSlice<'r, T>` holds an **exclusive** loan on the same range and additionally writes its elements.
Each is an `own` value carrying one loan region `'r` [TYPE-2, PROV-1], each is loan-bearing and owns nothing [OWN-1, STOR-5], and the strength is a component of the type's name, so the two are distinct types under the exact identity [TYPE-5] performs and neither is admitted where the other is required.
The measure table gives both the same row [MSR-1]: `len_of` is the viewed extent, `cap_of` equals it, and `room_of` and `head_of` are exactly zero, so a view is one contiguous unwrapped range and never a window.
`Slice<'r, T>` is **copy** and `MutSlice<'r, T>` is affine [OWN-1]: a second copy of a shared view is a second shared loan on the same range, which [OWN-5] admits without limit, and a view owns nothing, so a copy of one duplicates no obligation; two exclusive loans on one range are what [OWN-5] refuses, so the writable view stays affine.
A shared view is therefore used bare and `move` on one is the [OWN-1] error every copy value's is.
Two consequences other rules read. A copy view is consumed by nothing, so its loan begins where its value is formed or copied and ends at that value's **last use** rather than at a consume [OWN-5]; and a commit at a place of either view type is [VIEW-4]'s refusal, because [LIV-2]'s copy admission would otherwise displace a live loan with nothing consumed.
Neither view is ever stored in a nominal field, an enum payload, or a run slot [STOR-5], and neither is a generic type argument [FN-2]; a view crosses a function boundary only as one direct parameter or one direct `own` result whose origins [FN-1] bounds.

[VIEW-2] Formation, and the loan the formed value holds.
Exactly two operations form a view, each written as one [OP-1] table row over one borrowed place and each declared as one [BLK-0] record, as the last paragraph of this rule states:
`slice_of` takes `&'r place` and produces `own Slice<'r, T>`; `mut_slice_of` takes `&uniq 'r place` and produces `own MutSlice<'r, T>`.
The written borrow decides the row: a `&uniq` operand to `slice_of` and a shared operand to `mut_slice_of` are each a hard error citing TYPE-5 at that operand's `atom`, naming the borrow the written row takes.
`'r` is the region the operand's borrow takes, written or elided exactly as [FORM-8] states, and `T` is the viewed place's element type; neither is a written type argument, and a written argument list on either row is a hard error citing OP-1 at the `call`.
**The formed value, not the argument borrow, holds the loan**: the argument borrow is an ordinary call-scoped temporary [OWN-6], and the loan the formation establishes on the origin place lives for the region `'r` [OWN-5].
The access the formation itself performs is the access its own strength names — one shared access for `slice_of`, one exclusive access for `mut_slice_of` — judged against the complete loan state at that point [OWN-5].
Two exclusive views of one place are therefore refused at the second formation, which is [OWN-5]'s ordinary conflict at the unique borrow the second formation takes; a shared view of a place a live exclusive view already views is that view's shared child reborrow and is admitted, with the parent frozen against element writes while the child lives [OWN-5]; and two shared views of one place are admitted without limit.
A named const is the `immutable-const` origin of a shared view [OWN-5, CONST-2] and is never the origin of an exclusive one: `mut_slice_of` over a named const is a hard error citing CONST-2 at that operand's `atom`.
The viewed domain of both rows is the **viewable** operand class: the two runs [BLK-1], `array<T, N>`, and `buffer<T>` [OP-1]. It is one domain for both strengths: a view is a view of storage, and nothing in this rule reads what that storage is made of.
A **view reached through its holder** is viewable by `slice_of` and by `slice_of` alone: where `h` is a borrow-mode binding whose referent type is `Slice<'r, T>` or `MutSlice<'r, T>`, `slice_of(&'c deref(h))` forms that view's shared child reborrow [OWN-6], on this rule's own sentence — a view of a view is a view of the same storage under the narrower loan, and nothing here reads what the storage is made of.
The child carries the parent's complete origin set and its range; its loan region is the one the operand borrow writes, which the parent's own region must outlive [OWN-10]; the parent may not write the elements it views while the child lives and resumes at the child's own last use [OWN-5]; and the four relations the formation publishes are the parent view's own measures, which is [MSR-1]'s view row instantiated at the parent.
`mut_slice_of` over a view holder is a hard error citing OWN-5 at that operand, because two exclusive loans on one range are what that rule refuses and a child of a view is a second view of the parent's own range.
Each row is one [BLK-0] declaration record whose operand is spelled `vector` and whose mode is the borrow its own strength names, so the record's requirement and relation lists are what the formation submits and publishes.
Its one declared requirement is the **non-wrap premise** `head_of(vector) <= room_of(vector)`, which is `head_of(vector) + len_of(vector) <= cap_of(vector)` under [MSR-2]'s standing identity `len_of + room_of = cap_of` and is therefore an ordinary difference bound between two terms [ENT-4]; it is submitted at the formation and discharged under [MSR-4] exactly as every other row requirement is, and a formation whose operand does not discharge it is the ordinary [BLK-0] rejection naming the row.
A view is one contiguous range and a wrapped window is two, which is what that premise buys: an empty run discharges it from the standing `head_of <= cap_of` alone, so a drained ring is viewable, and `array<T, N>` and `buffer<T>` discharge it from their own measure-table row, whose `head_of` and `room_of` cells are both exactly zero [MSR-1].
Its four declared relations are the formed view's own measures: `len_of(result) == len_of(vector)`, `cap_of(result) == len_of(vector)`, and `room_of(result)` and `head_of(result)` exactly zero, which is [MSR-1]'s view row instantiated at the operand.
Both rows keep their [OP-1] table spelling over this domain, because two declaration domains may not claim one spelling [TYPE-6]; moving both spellings into the kernel IDENT domain is DEFERRED with recorded delta [META-5]: numbered rules +0, grammar productions +0, writer operation spellings -2, kernel declaration records +0. That spelling change does not require retirement of `array<T, N>`.

[VIEW-4] A commit may not displace a live loan.
A commit whose target place has loan-bearing type [VIEW-1] is admitted exactly when the displaced value is consumed by that same statement's right-hand side [LIV-2].
`set p = e;` where `p`'s selected type is `Slice<'r, T>` or `MutSlice<'r, T>` is therefore a hard error citing VIEW-4 at the complete target `place`, carrying that type and the restructuring `bind a new view under a new let rather than committing at this one`: [LIV-2]'s first condition would otherwise admit the commit at a copy target with nothing consumed, and the displaced view's loan would outlive the descriptor whose place it was held from.
`let old = replace p = e;` at such a place is refused on the same terms; where [SET-2]'s region-free target class already refuses that form, [DIAG-1] gives the citation to the earlier-defined rule and the program is rejected either way.
A commit at a place of any other type is untouched by this rule.

[VIEW-6] A view result declares its origin, and two results may not share one region.
The slice-result ceiling [FN-1] applies unchanged to each view type: a function whose written result is `own Slice<'r, T>`, or `own MutSlice<'r, T>`, has the ceiling containing `immutable-const` and the formal-view origin of every parameter whose written mode and type are exactly that same view type at the same formal region and element type.
A **shared** view result has in its ceiling additionally the formal-view origin of every parameter of borrow mode whose referent is a view at that same formal region and element type, at either loan strength: that origin is exactly what the shared child reborrow of a handed-on view carries [VIEW-2, OWN-6], so a helper handed `&uniq MutSlice<'r, T>` fills that destination, forms the child, and returns it, which is what makes the fill-and-publish helper writable.
No exclusive result takes this half: the only child a borrowed view holder can form is shared [OWN-5], so no borrow-mode parameter is ever a source of an exclusive one.
An ordered result list containing two results of the same view type at the same formal region is a hard error citing VIEW-6 at the `result_binding` of the second, with the restructuring `give each result its own formal region`: without it a demux written with one region returns views each of which the ceiling says may alias every input.

## 6. Storage

[STOR-1] Storage class is a function of type, stated once: `box<T>` is heap-owned; `arena<'r, T>` is arena-owned, bounded by `'r`; `buffer<T>` is heap-owned (one compiler-derived heap allocation, released by one compiler-derived free at owner scope-exit [STOR-3]); `Vector<'s, T>` is store-owned, its one run of slots taken from the store `'s` names and released to that store [PROV-1, BLK-1]; `Box<'s, T>` is store-owned in exactly the same sense, its one cell taken from that store and released to it [S39]; `FixedVector<T, n>` is frame-resident, its slots inline in its owner or the stack frame; `Arena<'s, bytes, align>` is the extent `'s` names, laid out in the reserving activation's frame [BLK-2]; `Heap<'s>` has no storage of its own, being an ordinary proof-only provider parameter [PROV-1]; a `const` item [CONST-2] is immutable static storage (program-lifetime, read-only, never dropped); every other owned value is frame-resident (inline in its owner or the stack frame).
There is no per-binding storage annotation and no default clause.
An owned `array<T, N>` is frame-resident, inline in its owner or the stack frame: exactly N stride-spaced element representations in index order and no per-array length, capacity, head, occupancy, or discriminant storage.
Its concrete size, stride, padding, and zero-extent representation obey [STOR-6]; placing the array inside another owner changes no element order or ownership.
SET-1 may overwrite a copy-typed final place, and an affine one exactly where [LIV-2]'s first condition admits it; [SET-2] may replace only a region-free affine final place, binding the previous owner.
Setting a live affine-typed final place with `set` where the right-hand side does not read its previous value out is a hard error citing STOR-1 at the complete target `place`, carrying its exact affine type and the restructuring `use replace: let old = replace p = e; binds the previous owner`.
This specification defines no bare take operation, temporary uninitialized hole, vacancy type state, or implicit destruction of the old affine value: [SET-2]'s atomic exchange and [LIV-2]'s read-out-and-commit are the two affine replacement forms, and in both the previous value leaves through a written destination before the new one arrives.
Growable or keyed collections (dynamic vector, hash map, set, byte-string, text) are neither storage classes nor kernel constructs: they are library structures over `buffer<T>` plus struct/enum and generics (a byte-string is `buffer<u8>`; a growable vector pairs a `buffer<T>` with a length, growing by allocate-new, move, [SET-2] field replace, and ordinary release of the superseded buffer).
The arena-index-pool ownership pattern remains rejected as a collection basis (it resurrects use-after-free as well-typed slot-recycling); keyed collections additionally remain blocked on their own occupancy and identity designs.
Char and Unicode text are out-of-v0, recorded.

[STOR-2] Creation: `box_new(v)` returns `own box<T>` for `v`'s exact type T [OP-2]; `arena_new::<'r, T>(v)` returns `own arena<'r, T>`; both are ordinary calls in the operation table.
Content access is through `deref`.

[STOR-3] Deallocation and resource release are compiler-derived and explicit in the checked program [DIAG-2]: every drop and every release is represented before lowering.
Every control-flow edge leaving a region block (fallthrough, `break`, `return`) carries that region's releases and drops in reverse declaration order.
Release actions run on every source control-flow edge that leaves their owner scope.
Host termination caused solely by unavailable external resources under [SCOPE-3] is not a Whitefoot control-flow edge, and this specification makes no source-level cleanup promise for that deferred case.
No reference counting.

Every edge that leaves one entered `for_stmt` body normally — its fallthrough, a `break` resolved to that counted loop or an enclosing loop, a `return`, or a `propagate` error edge — carries exactly once every compiler-derived drop and release for the body scopes that edge leaves, innermost scope first and in reverse declaration order within each scope.
On body fallthrough those actions complete before the hidden counted update [FN-1].
The header's false edge never enters the body and therefore carries no body-scope cleanup.
An external-resource termination under [SCOPE-3] likewise creates no source edge on which these actions could run.
No exit duplicates an action already carried by an inner scope edge.

The release action of a type is compiler-owned semantic data selected by that type, not a fixed enumeration of memory-reclamation actions.
Which components of a value that action visits, and in which order, is [PROV-6]'s release graph and its one walk; the per-type actions below are the leaves that walk runs, and `dispose p;` runs the same walk earlier [PROV-6].
A `box<T>` drop is one compiler-derived heap free.
A `buffer<T>` drop with copy-typed elements is one compiler-derived heap free on every owner-scope exit, ordered like a `box<T>` drop.
A `buffer<T>` drop with affine-typed elements [TYPE-2] is each element's compiler-derived drop in ascending index order followed by that same one heap free; for an element type whose own drop derives no action, the composite action remains exactly the heap free.
An `arena<'r, T>` value's storage is released with its region [STOR-4].
A `Vector<'s, T>` drop is each element's compiler-derived drop over its window in ascending logical index order [BLK-1], followed by one compiler-derived release of its run to the store `'s` names; that release is the store's own reclamation and spends that store's provider capability [PROV-6].
An `Arena<'s, bytes, align>` value has no release action: its extent is the reserving activation's own frame storage, its initial state is established at the reservation on every activation of `'s`'s region block, and nothing of it is observable outside that block [BLK-2].
A `Box<'s, T>` drop is its referent's compiler-derived drop followed by one compiler-derived release of its cell to the store `'s` names, which spends that store's provider capability where the store is a general one and is empty where `'s` is a bump extent, that extent's reclamation being its own region reset [PROV-6, S39].
A `Heap<'s>` value has no release action; its ordinary region and capability rules govern every value backed by that store [PROV-1, PROV-6].
A `FixedVector<T, n>` drop is each element's compiler-derived drop over its window in ascending logical index order and no storage reclamation of its own, its slots being frame-resident [STOR-1].
An `array<T, N>` drop is each of its N elements' compiler-derived drops in ascending index order and no storage reclamation of its own; at N equal to zero it executes no element action [PROV-6].
A `const` item [CONST-2] is never dropped.
Every other frame-resident owned value [STOR-1] has no release action.
Each of these memory-reclamation actions carries the empty effect row exactly when its walk spends no capability, and otherwise carries `writes` of each provider place the walk resolves [PROV-6, EFF-2].

An opaque nominal's drop is empty. Its declaration's `linear` modifier, and only the ordinary ownership closure of [PROV-6], requires explicit consumption.
No source declaration, annotation, attribute, contract, or binding attaches a finalizer or any other user-defined action to a value's drop.

A successful SET-1 assignment derives no drop, release, finalizer, or cleanup edge: a copy target's previous value needs none, and an affine target's previous value has already left through [LIV-2]'s read-out or was already gone, every other affine target being rejected before checked-program construction [STOR-1].
A successful [SET-2] commit likewise derives no drop, release, finalizer, or cleanup edge: the previous value is not destroyed, and its later release, if its binding is abandoned, is that binding's ordinary scope-exit action under this rule.

[STOR-4] Arena confinement: a value of type `arena<'r, T>` may not be returned, stored into a field, or moved to a destination outside `'r`'s block; borrows of its content obey OWN-10 with source region `'r`.

[STOR-5] Storage is borrow-free and region-free, and a store brand is neither a borrow nor a confinement.
A type is region-bearing when its complete type after generic substitution contains `Slice<'r, T>`, `MutSlice<'r, T>`, `arena<'r, T>`, `Heap<'s>`, or `Arena<'s, bytes, align>` at any depth: a loan whose provenance the storage would hide, a value the region's own release reclaims, and a provider a move would strand.
A store-branded run is **not** region-bearing under this relation even though its type names a region, because the store's identity travels in the type [PROV-1]: the position is itself confined to that store, and nothing about the value outlives, hides, or strands anything.
No struct field, enum variant payload, `array`/`buffer`/run element, or `box`/`arena` content may be a borrow or a region-bearing type; a `Vector<'s, T>` is admitted in every one of them, which is what makes `struct N['s] { field: Vector<'s, T>; }` and `Option<Vector<'s, T>>` well-formed, and a provider type in any of them is a hard error citing STOR-5 at the complete contained `type`, with the restructuring `keep the slice, arena, or provider as a direct local, parameter, or result; do not store it inside another value`.
The confinement judgment over an admitted store-branded position — that the owning value's own complete type names each region the stored value's does — is [BLK-4]'s; the stored-position and loan-bearing judgments do not prohibit exclusive parameters whose referents reach runs or generic type parameters.
The `field`/`vfield` grammar admits only `type`, and `type` has no borrow (`&` / `&uniq`) production [GRAM-3]; the semantic check is recursive after substitution and therefore also closes indirect forms such as `box<Slice<'r, T>>`, `arena<'a, Slice<'r, T>>`, and a generic field instantiated with a region-bearing type.
A violation is a hard error citing STOR-5 at the complete contained `type` whose placement would make storage region-bearing, with the restructuring `keep the slice or arena as a direct local, parameter, or result; do not store it inside another value`.
A direct view type [VIEW-1] or `arena<'r, T>` remains a legal complete parameter, local, or result type where its owning rules admit it.
Substituting a region-bearing T into `box_new` or `arena_new<'a, T>` places T in an enumerated `box` or `arena` content position and therefore rejects under STOR-5: for `arena_new`, at the complete `type` child of that operation call's `targ`; for `box_new`, whose content type is derived from its operand [STOR-2, OP-2], at that operand `atom` node and its complete checked half-open source extent.
A `slice` element is not one of this rule's enumerated stored-content positions: writing `Slice<'s, Slice<'r, T>>` does not by itself violate STOR-5 and retains its v0.16 type-formation status.
The ordinary `slice_of` source path cannot construct that value because its array or buffer source would place the region-bearing inner slice in an element position prohibited above. [FN-2] separately rejects a loan-bearing or provider generic type argument at its source `targ`, and admits a store-branded one on this rule's exception.

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
The accepted [OP-9] judgment retains a numeric upper bound for the source length at that allocation site; target qualification multiplies that bound by the actual target stride using checked mathematical arithmetic and requires the result to fit both the allocator-parameter and address-index domains before lowering the operation.
At this target stage, the exact SSA result of `len_of(buffer)` additionally carries the selected target's runtime-allocation byte maximum divided by that buffer's actual element stride, because every materialized buffer already satisfies the successful-allocation representation invariant.
Qualification may intersect this target bound with the retained source bound only for that exact SSA result; it does not publish a Whitefoot comparison fact or transfer the bound through a block parameter, storage load, conversion, user call, or another value merely because its source spelling or type is similar.
The source allocation proof and this target qualification jointly establish that every reachable runtime byte count has one exact value-preserving target representation; neither alone authorizes emission, and the allocator receives exactly that value.
Every emitted target address computation must likewise be proved valid for every runtime value that reaches it: the compiler establishes before emission that each runtime index and each mathematically scaled byte offset actually used by the computation has an exact value-preserving representation in the applicable target address-index domain, and that scaling and offset addition do not wrap.
An [OP-4] bounds judgment together with an established complete-object-layout or successful-allocation invariant may discharge these obligations; a backend's implicit narrowing does not.
If target qualification cannot establish one of these facts, target compilation stops before emitting the governed allocation or address operation.
This stop is a target-layout failure, not a source rejection, [OP-4] bounds failure, runtime proof outcome, or resource-availability failure; no target-domain runtime guard is emitted.

Complete generated frames remain subject to the mandatory checked-representability judgment above.
That judgment does not predict available stack capacity: available capacity depends on dynamic call depth, recursion, the caller, and the execution environment.
The language therefore defines no numeric per-array, per-object, or per-function frame ceiling.
A tool or selected target may stop compilation for its own conservative frame-capacity or resource limit as a non-language target/resource failure [DIAG-1], but that optional limit does not replace the mandatory representability judgment.
Exhaustion during execution is inside the compiler/runtime/OS TCB boundary [SCOPE-3]: it adds no source effect or proof fact and authorizes no hidden heap promotion.

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
| `==` `!=` `<` `<=` `>` `>=` | all int T | `(T, T) -> own Bool` | pure |
| `eeq` `ene` | one exact nominal tag-only enum T (every variant nullary), including `Bool` | `(T, T) -> own Bool` | pure |
| `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` | f32 f64 | `(T, T) -> own T` | pure |
| `feq` `flt` `fle` `fgt` `fge` `fne` | f32 f64 | `(T, T) -> own Bool` | pure |
| `band` `bor` `bxor` | Bool | `(Bool, Bool) -> own Bool` | pure |
| `bnot` | Bool | `(Bool) -> own Bool` | pure |
| `cvt` | value-preserving pairs [OP-6] | `(Src) -> own Dst` | pure |
| `cvt` | all other distinct numeric pairs [OP-6] | `(Src) -> own Result<Dst, NarrowError>` | pure |
| `len_of` `cap_of` `room_of` `head_of` | `Slice<'r, T>`, `MutSlice<'r, T>`, `array<T, N>`, `buffer<T>`, `FixedVector<T, n>`, `Vector<'s, T>`, `Arena<'s, bytes, align>` | `-> own u64` | pure |
| `slice_of` | `array<T, N>`, `buffer<T>` | `&'r place -> own Slice<'r, T>` (a borrow of the whole array/buffer place) | pure |
| `mut_slice_of` | `array<T, N>`, `buffer<T>` | `&uniq 'r place -> own MutSlice<'r, T>` (a unique borrow of the whole array/buffer place) | pure |
| `box_new` | any T | `(own T) -> own box<T>` | allocates(heap) |
| `arena_new` | any T | `(own T) -> own arena<'r, T>` | allocates(arena 'r) |
| `array_new` | `T` copy (v0: primitive), `N` a constant-expression [CONST-1] | `(T) -> own array<T, N>` (fills all N elements with the argument) | pure |
| `buffer_fits` | `T` a concrete region-free buffer-storable type [TYPE-2, OP-9] | `(u64) -> own Bool` | pure |
| `buffer_new` | `T` copy (v0: primitive) | `(u64, T) -> own buffer<T>` (allocates a flat buffer of the u64 length and fills every element) | allocates(heap) |
| `buffer_vacant` | `T` region-free [STOR-5] | `(u64) -> own buffer<Option<T>>` (allocates a flat buffer of the u64 length; every element is `None()` of `Option<T>`, compiler-minted, no source value duplicated) | allocates(heap) |
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
An `infix_op` or `compare_op` token resolves to its exactly spelled operation by the operator table row; infix resolution consults no name domain, and an operator token is never a declaration, callee IDENT, or OPNAME.
An IDENT callee whose spelling belongs to `DotlessOperationNames` resolves to that operation family; every other IDENT callee admits an in-scope raw function-kind parameter [FN-5], a top-level source `fn_decl`, or a PRE-1 function.
Absence from the selected operation-family, function inventory is a hard error citing OP-1.
Later typed operation checking uses the operand domains and, for the retained-argument operations [TYPE-5], the written arguments, to select the applicable row within the resolved family.
Operand types never select between an operation family and a function.
A bare `place` operand that a table-operation row reads without consuming — the operand of each measure former `len_of`, `cap_of`, `room_of`, and `head_of` [MSR-1], the place viewed by `slice_of` through its explicit borrow, and the base place of a subscript — is a non-consuming read: it neither moves nor partially consumes an affine root [OWN-1], exactly the reading [FN-8] already states for a place used as a non-consuming operand of an admitted table operation.

No source declaration or FN-9 result-datum candidate in this closed list may use a member of `ReservedLowerNames`: the IDENT of `fn_decl`; the IDENT of `const_decl`; every `param` and `result_binding` IDENT; every `let_stmt` IDENT, including ordinary, propagate, value-match, and value-if lets; every `contract_define` IDENT; the second IDENT of any `fieldbind`, including a `result_route` payload binder; every `field` and `vfield` IDENT; and the IDENT-shaped interior of `region_params` and `region_stmt`.
Such a reserved spelling is rejected citing exactly FORM-3 before freshness ownership is considered.
Dependent field declarations participate in this pre-resolution reservation inventory even though their owner/member duplicates remain deferred.
No other declaration role is covered: type-generic TYPEIDs, const-generic IDENTs, LABELs, and formal-member `fn_sig` IDENTs remain outside this prohibition.
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
An executed branch condition, proved requirement, proved invariant, or verified postcondition may establish its canonical goal through [ENT-3].
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

For `==`, `!=`, `<`, `<=`, `>`, and `>=`, both operands denote their mathematical values in T and the result is respectively `True()` exactly when `a=b`, `a!=b`, `a<b`, `a<=b`, `a>b`, or `a>=b`.
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

[OP-4] A subscript `p[i]` selects one element place of an indexable base: the base place `p`'s final selected type must be `array<T, N>`, `Slice<'r, T>`, `MutSlice<'r, T>`, `buffer<T>`, `FixedVector<T, n>`, or `Vector<'s, T>` [BLK-1, VIEW-1], and the subscripted place's selected type is exactly that element type T — derived from the base place's already-fixed type [TYPE-5] — written where the binding carries an annotation, derived at a body `let` — by the same declared-type selection that types a field suffix, never from expected type or cross-statement inference; a subscript whose base's final selected type is not one of the six indexable types is a hard error citing OP-4 at that subscript's `psuffix` node.
The subscript carries the bounds obligation `i < len_of(p)` [ENT-6], and `i` is a logical offset [MSR-1]: it names the slot at physical offset `(head_of(p) + i) mod cap_of(p)`, so the obligation is against `len_of(p)` for every indexable base and never against `cap_of(p)`.
The injectivity sentence of [MSR-1] is what carries that logical conclusion to a storage conclusion, and its premise `len_of(p) <= cap_of(p)` is one of [MSR-2]'s standing facts, so no subscript occurrence submits a separate obligation for it.
The obligation is submitted to the one numeric goal disposition [MSR-4]; that rule fixes the complete ordered derivation and this rule grants no route of its own.
A discharged subscript reads or writes with no runtime bounds check in every build mode, and its checked-program disposition records the discharging derivation [DIAG-2].
A subscript whose current ProofContext does not discharge the obligation is a compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying the residual obligation rendered exactly per [ENT-6], and publishes no checked program.
Its mechanical fix is a dominating branch establishing the residual [ENT-3], a proved header or local invariant [INV-1], an invariant carrying sufficient [PRF-1] uses, or a verified callee relation [FN-9].
Discharge is a deterministic checker derivation [ENT-1]; a solver result never participates.
A `buffer<T>` obligation is over the runtime length term.
The offset atom has exact value mode and type `own u64`; after the [TYPE-7] implicit-read exclusivity, any other offset mode or type is a hard error citing OP-4 at the offset `atom` node, with `SourceCoordinate` equal to that atom's complete checked half-open source extent.
A subscript in a [SET-1] target forms the selected place without reading its stored value; its base and offset are evaluated during target evaluation, and its discharge judgment is identical in target position.
A successful bounds judgment neither narrows nor authorizes narrowing the offset or its scaled byte offset; target address formation additionally obeys [STOR-6].

[OP-5] Every source condition and contract predicate requires its selected expression to have exact value mode and type `own Bool`, where `Bool` is the PRE-1 nominal type.
No integer, other enum, borrowed `Bool`, or implicit truthiness conversion is admitted [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent `Bool` value would be required, that use is rejected citing TYPE-7 and OP-5 forms no candidate.
Every other exact-mode or exact-type failure is a hard error citing OP-5 at the selected `expr` node, with `SourceCoordinate` equal to that expression node's complete checked half-open source extent.
An `if` condition is executed control flow [GRAM-6], while a contract predicate, invariant relation, and `proof_use` are erased proof syntax [FN-8, FN-9, INV-1, PRF-1].
This judgment alone creates no runtime check or effect.

[OP-6] cvt partition and semantics (cross-reference TYPE-4).
`cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value is exactly representable in Dst (y the unique such Dst value) and `Err(NarrowError())` otherwise, and it never rounds, truncates, or saturates.
A non-integral float-to-int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or infinity targeting an integer all yield `Err`; for float-to-float, an infinity maps to the same infinity and NaN maps to the target canonical quiet NaN (value-preserving).
A pair is TOTAL — signature `(Src) -> own Dst`, no Result — where every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`.
Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.

[OP-7] Operation-name convention.
An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len_of`, `cap_of`, `room_of`, `head_of`, `slice_of`, `mut_slice_of`, `box_new`, `arena_new`) carry no prefix.
The integer arithmetic and integer comparison symbols of [GRAM-5] are the one prefix-free operation class: each is an integer-only table row, so `+` and `<` never denote a float or enum operation, and `fadd.strict`, `feq`, and `eeq` keep their prefixed names.
`Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family.
A respelled operation's token is its one constant spelling under the same one-spelling-per-operation discipline.
Bare infix and dotless named integer spellings are proof-required exact operations; `.defined` is the distinct total Bool-valued domain query, not a result mode and not an execution of the partial primitive.
The total value-result policies remain `.wrap`, `.checked`, and `.sat` where [OP-1] lists them, and float `.strict` is unchanged.
Signedness-parametric lowering keyed on the operand-derived selected type [OP-2] (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `<` = `slt`/`ult` row, not overloading.
Nominal enum identity is likewise checked from the operand-derived selected type before `eeq`/`ene` lowering; equal representation width never makes distinct enum types interchangeable.

[OP-8] Edge semantics and confirmed lowerings for the operations added in this revision; every totality edge is closed here as table data, so no added row is writer-reachable poison.
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
`finf` is the positive-infinity value (negative infinity is `fneg(finf::<T>())`) and `fnan` is the canonical quiet NaN; other NaN payloads are reachable through `reinterpret`.
For a tag-only enum T — the operand-derived selected type [OP-2] — `eeq(a, b)` is `True()` exactly when `a` and `b` denote the same declared variant of that nominal T, and `ene(a, b)` is its exact boolean complement.
Both operands must have that exact T, derived by [OP-2]'s agreement rule; representation equality never permits cross-enum comparison.
`Bool` is admitted by the same tag-only rule.
Both operations lower directly to equality or inequality of the validated discriminants in T's already-selected representation.
They are pure and total: after normal operand evaluation, the primitive does not inspect a payload, access memory, trap, convert a value, or introduce a new optimizer fact channel; an operand read still exhibits its ordinary effect before the primitive executes.
Payload-carrying enums, enum ordering, and enum/integer conversion remain outside the operation table.

[OP-9] `buffer_fits::<T>(n)` is the pure, total, target-independent allocation-domain predicate
`n <= floor((2^64 - 1) / stride_ceiling(T))`, where `stride_ceiling(T) >= 1` is the language layout ceiling fixed below.
It returns `own Bool`, exposes no target ABI value, and has the same result for one source type and n on every qualified target.

`buffer_new(n, v)` over fill type T carries the one canonical obligation `buffer_fits::<T>(n)`.
`buffer_vacant::<T>(n)` carries `buffer_fits::<Option<T>>(n)`.
Each acquiring kernel-domain row [BLK-2] over element type T and count `n` carries the same predicate over `(T, n)`, which is the obligation [BLK-0]'s record notation spells `fits::<T>(n)`; the predicate, its normalization, and its disposition are this rule's and are the same object under either spelling.
Each is accepted only when [ENT-6] discharges that exact goal; its sole normalized component is the defining comparison above, which may supply an alternate L0 derivation of the same root.
The root does not project a new general L0 fact in the other direction.
A refuted or unproved goal is a static OP-9 rejection; a contradictory state discharges it under [ENT-4].
When n comes from runtime input, it remains an ordinary symbolic term; only an enumerated fact constructor such as a selected real branch, a proved invariant target (including one checked by [PRF-1]), or a verified postcondition may discharge this goal [SCOPE-2, ENT-3, ENT-6].
No written conclusion alone, runtime multiplication guard, or fallback is retained.

All layout-ceiling arithmetic is over unbounded mathematical integers.
Let `round_up(x,a) = ceil(x/a) * a`.
For a sequence of `(size, alignment)` pairs, start at offset zero, round each current offset up to the next field's alignment, add that field's size, take aggregate alignment as the maximum of one and the field alignments, and round the final offset to that aggregate alignment.
The primitive `(size_ceiling, align_ceiling)` pairs are: `unit`, `Bool`, `i8`, and `u8` `(1,1)`; `i16` and `u16` `(2,2)`; `i32`, `u32`, and `f32` `(4,4)`; `i64`, `u64`, and `f64` `(8,8)`; `box<T>` `(16,16)`; `buffer<T>` `(32,16)`; every opaque nominal `(32,16)`; `Vector<'s, T>` `(32,16)`, a descriptor of pointer, capacity, length, and window origin; and `Heap<'s>` and `Arena<'s, bytes, align>` `(32,16)`, a proof-only representation whose only stored component is the store's own cursor state.
A struct applies the sequence rule to fields in declaration order; an array repeats its element pair N times.
A `FixedVector<T, n>` repeats T's pair `n` times and then applies the sequence rule to that block followed by two `(8,8)` descriptor words, its length and its window origin, so its aggregate alignment is `max(align_ceiling(T), 8)`; its capacity is the type constant and is stored nowhere [MSR-2].
A tag-only enum with at most two variants has `(1,1)`, and every other tag-only enum `(4,4)`.
A payload enum, including `Option` and `Result`, sequences a `(4,4)` tag followed conservatively by every variant payload field in variant and field declaration order.
Region-bearing slice and arena types are outside `buffer_fits`'s domain; existing recursive-type rejection remains, while box, buffer, and `Vector` ceilings do not recursively expand their content, and a provider type is outside the allocation-fit domain because no operation acquires a run of providers.
`stride_ceiling(T)` is `max(1, size_ceiling(T))` after the aggregate rule.

Before emitting a stored type S, target qualification verifies that its actual size, alignment, and stride do not exceed the three language ceilings.
Only with both that qualification and the source obligation disposition may lowering emit `n * actual_stride(S)` as non-overflowing arithmetic.
Qualification failure is a target failure and may not become a runtime guard.
The [STOR-6] rule separately governs allocator and address-index representability; allocation failure remains a TCB/resource failure [SCOPE-3], never a language trap.
`array<T, N>` performs no runtime size computation: N is fixed at monomorphization and concrete target representability is checked under [STOR-6].
The language defines no numeric frame limit, and `array_new` remains pure because target-layout and resource failure are not program execution.

## 8. Functions, generics, contracts

[FN-1] A concrete function's callable boundary states everything ordinary callers need: parameter modes and types, the ordered result list's modes and types, one formal-path state-effect row, its region parameters and the unnamed regions of its remaining region positions [FORM-8], the ordered [FN-8] requirement GoalTemplates, the ordered verified [FN-9] normal-result RelationTemplates.
Every result binder's spelling is mandatory but ignored by callable-signature equality and denotes no runtime storage.

A `fn_decl` or `fn_sig` writes one result or a parenthesized list of two or more [GRAM-2].
Each `result_binding` is one **result ordinal**, numbered from zero in written order; the list's binder spellings are distinct under [TYPE-6], and each ordinal receives every result judgment of this rule independently — an `own slice` ordinal has the same parameter-derived return-origin ceiling, and a borrow-mode ordinal the same signature-determined provenance.
A declaration that writes a list hands its ordinals back together, and a caller names them again only through a destructuring `let` binder list or a `set` target list [GRAM-4, TYPE-5, CALL-4]; no expression position produces a result list, so a list-returning callee is bindable only by those two forms.
This rule's remaining sentences are stated over a written result and read per ordinal where a declaration writes a list.
The written templates are ordinary interface propositions. A Whitefoot definition proves them under FN-9; a PRE-1 definition is supplied under SCOPE-3. A caller consults only the declared finite summary and never the definition.
The written effect paths state which parameter-supplied state the function observes or changes. The checker derives the exact same set from body accesses, ordinary memory reclamation, and calls and checks it in both directions under [EFF-2].
Strengthening a requirement GoalTemplate or RelationTemplate is a caller-visible interface change.
A generic function carries the same boundary with its written type, const, and function parameters, and each concrete [FN-2] instance substitutes them before its calls and body are re-checked.
A `fn_sig` may carry the same requirement and postcondition templates. FN-4 checks their formation and structural equality at binding; its selected ordinary definition supplies their proof under FN-9 or PRE-1.
Function-signature visibility is the [TYPE-6] table.
Every explicit `return e1, ..., en;` writes exactly as many expressions as the enclosing declaration writes results, and expression i must produce exactly result ordinal i's `rtype`; there is no result-mode or result-type conversion [TYPE-4].
A written count other than the declared result count is a hard error citing FN-1 at the `return_stmt` node.
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent value would be required by the written `rtype`, that use is rejected citing TYPE-7 and FN-1 forms no candidate.
Every other return mode or type mismatch is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child.
FN-9 adds a stricter result and return-expression shape only for a function that declares an `ensures_clause`; a function with none retains every return form admitted here.

For a function whose written result is `own Slice<'r, T>`, the written signature also determines one return-origin ceiling without additional syntax.
The ceiling contains `immutable-const` and the formal-slice origin of every parameter whose mode and type are exactly `own Slice<'r, T>` denoting that same formal region and element type; an elided parameter region denotes a distinct region [FORM-8] and therefore never supplies the result.
No parameter with a different mode, type, element type, or formal region is a supplier.
In particular a borrow-mode parameter and an `arena<'r, U>` parameter are not implicit slice suppliers.
Every explicit `return e;` producing that written result must have an [OWN-5] origin set contained in the ceiling.
Failure is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child and the restructuring `accept an exact direct input slice in the result region or keep the newly formed view in its caller; do not return a view of raw callee storage`.
OWN-10 independently rejects a returned origin whose storage is too short-lived.

A function whose written result mode is `&'d` or `&uniq 'd` and whose direct result type is `Slice<'r, T>` is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor`.
This specification has no signature summary that carries both the returned descriptor's source-place provenance and the underlying slice value's complete origin set.
This rejection does not change any other returned-borrow judgment.
A function whose result mode is `&'b` or `&uniq 'b` determines the result's provenance from its written parameters alone: a parameter is a provenance candidate iff its mode is a borrow of the result's kind in the result's formal region `'b` [OWN-6].
Because a candidate shares `'b`, [FORM-8] writes `'b` at both positions; a function with no candidate writes `'b` at its result alone and its caller supplies that region.
Exactly one candidate is the result's debtor, and zero candidates is legal — OWN-10 admits no `'b`-region borrow rooted in callee-local storage, so the only remaining source is named `const` storage, whose immutable program-lifetime extent needs no caller loan [CONST-2].
The provenance judgment applies to a result whose written type is region-free; a region-bearing result type is rejected before it — a direct slice by this rule's slice sentence and an arena, in either result mode, by [STOR-4].
Two or more candidates, a same-region parameter of the other borrow kind, or any parameter whose type carries `'b` leaves the source undetermined and is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `give the source parameter its own region so exactly one parameter shares the result's region and kind, or return the decision as a value and let the caller borrow from the source it names`.
The declaration is the error and no call is required to reach it: [GRAM-9] admits a computed value only through a preceding `let`, so a result no caller can bind is unusable by construction.

The signature-formation parts of these two slice-result judgments and of the borrow-result provenance judgment apply equally to a top-level `fn_decl` and a function-formal `fn_sig`: an `own slice` member has the same parameter-derived ceiling, a borrow-mode direct-slice member is rejected at that member's complete `rtype`, and a borrow-result member whose source its own parameters leave undetermined is rejected there too.
A `fn_sig` has no body returns to validate. A FN-3 binding requires the bound ordinary declaration to satisfy the same signature judgment; a bound Whitefoot definition independently satisfies the complete body judgment.

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
The counted header's carried-identity set is exactly the bindings carried into the construct plus both captures and the binder; the continuation interface and a break resolved to that counted loop carry only the incoming identities after their path-specific ownership, cleanup, and effect judgments, with any counted label, the binder, and captures all out of scope.
Ordinary `loop_stmt` execution is unchanged.

Function completion and statement reachability use one conservative structural normal-control graph over the resolved function body.
For any statement s, `normal_successor(s)` is the entry of s's next sibling statement in the same block when one exists, and otherwise that containing block's normal exit.
A block entry reaches its first statement, or its normal block exit when it contains no statement.
An ordinary `let`, a `let` selecting `replace_let_rhs`, `set`, an expression statement, and an `invariant_stmt` have a normal edge to `normal_successor(s)`; an `invariant_stmt` is then erased before lowering.
A call with a normal result edge never proves divergence merely because external resource availability is outside this cycle's guarantee [SCOPE-3].
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
A `break` resolved to the counted loop reaches `normal_successor(for_stmt)` without the update.
Every counted header retains both structural edges even when its captured endpoints are constant, so no counted loop is assumed to execute or to diverge [GIVE-1].
The function-body normal exit has no successor.
These edges are structural and are not removed by constant evaluation, a proof, or backend reachability.

Each statement not reachable from function-body entry establishes an FN-1 rejection premise using `SourceNode` at the selected concrete statement production beneath its `stmt` wrapper and a `SourceCoordinate` equal to that production's complete checked half-open source extent.
When more than one statement establishes that premise, the reported one follows DIAG-1's implementation-defined deterministic traversal. [GIVE-1] remains the more specific owner of a statement following `give` in the same block, so that statement establishes no additional FN-1 reachability rejection.
The function body's normal exit must be unreachable.
If it is reachable, the function falls through and is rejected citing FN-1 at the `fn_decl` node, with `SourceCoordinate` equal to the complete source interval of the body-closing `}` token.
This requirement applies to `own unit` as well as every other result: successful completion is written `return unit;`; there is no implicit return.
A call with no termination proof or a loop does not satisfy the return requirement.
This complete structural graph, its statement reachability, and every source call and invariant-declaration identity are retained for source audit even when [FN-8] later proves one concrete instance uninhabited.
That proof changes only its checked body disposition and lowering authority; it never erases a source node or narrows the written effect row.

[FN-2] Function and nominal generics are monomorphization-only; type, const, and function instantiation arguments are always explicit and region arguments are written exactly where [FORM-8] writes them. FN-3's named groups abbreviate those explicit parameter and argument vectors; expansion is compiler-side, pre-IR, and instantiations are re-checked as concrete code.
Every contract definition and requirement or postcondition template is substituted separately for each concrete function instance.
The [FN-8] uninhabited judgment is likewise instance-local and never propagates from one concrete substitution to its generic template or another instance.
The region arguments one call determines — those its operands fix and those [FORM-8] makes it write — are substituted into every position of the callee's signature at that call, every output position exactly as much as every input position, and at any depth of each: through a PRE-1 nominal's arguments, into a source nominal instance's own region arguments [TYPE-2], into a run's element position [BLK-1], and into every ordinal of a declared result list [FN-1, CALL-4].
A result therefore carries the region that call fixed and never the declaration's own formal region, so two calls of one declaration whose operands name two stores hand back two types under the exact identity [TYPE-5] performs [PROV-1]; a result that kept the formal region would let a run taken from one store be typed later as a run of another, which is the one thing that region is there to decide.
Every explicit type argument supplied to a function, source nominal, or PRE-1 nominal generic parameter must be free of a loan-bearing and of a provider leaf: it may not contain `Slice<'r, T>`, `MutSlice<'r, T>`, `arena<'r, T>`, `Heap<'s>`, or `Arena<'s, bytes, align>` at any depth after substitution.
Such an argument is a hard error citing FN-2 at that complete `targ`, with the restructuring `make the slice, arena, or provider a direct written parameter or result instead of a generic argument`; there is no generic substitution, storage, result, or call-summary rule for a hidden loan or provider leaf.
A store-branded argument — one whose regions are all store regions [PROV-1], `Vector<'s, T>` and every type over it included — is admitted, and the position it lands in is confined to those regions by [STOR-5]'s exception. Its complete opaque type identity retains every captured brand through function, nominal, and group substitution; a receiving declaration need not introduce redundant region parameters for brands hidden inside a type argument. Such hidden brands supply no additional FORM-8 inference positions, cannot be renamed to a different store, and do not extend any value's confinement or loan. A directly written branded type still writes and substitutes its ordinary region positions.
Arguments this rule admits remain governed by the ordinary bound and substitution rules.
A generic type parameter's numeric bound is admitted only when it resolves to the built-in `Int` or `Float` bound [PRE-1]. A formal or actual group is not a numeric bound; it occupies its own explicit argument-list position [FN-3].
A written `copy`, `affine` or `linear` bound on a type parameter, and a written `affine` or `linear` bound on a region parameter, is [PROV-6]'s linearity class, written and never inferred, read once at the declaration and checked at every instantiation by that rule. It selects no behavior; function-kind parameters supply behavior separately.
Every type parameter of a function or of a nominal carries exactly one bound, written and never inferred, with no default: one marker TYPEID [FN-3] or one of the three linearity classes [GRAM-2, PROV-6]; a marker bound is a numeric row [OP-1] and implies the copy class.
The template is the spelling authority: a generic body is checked once at the symbolic instance of its own parameters, under each parameter's written bound, and the concrete-instance recheck this rule performs does not re-judge the spellings [FORM-1] keys on a value's copy/affine class — `move p` against a bare `p` [OWN-1] and `replace` against `set` [SET-1, SET-2].
At a concrete instance a `move` of a value whose parameter was bounded `affine` or `linear` denotes a copy where the argument is copy, and a `replace` of such a value denotes the same exchange; every other judgment of those rules is made at the instance, because each is a property of the instance and not of the written spelling.

[FN-3] A function-kind generic parameter is one `fn_sig`: an ordered ordinary callable signature with its own regions, effect row, requirements, and ensures. It is a compile-time parameter, never a value, field, receiver, or implicit argument.
A `formal` declaration names one ordered parameter group. Its header declares type and const parameters with their ordinary bounds; its body declares the function-kind parameters in source order, with distinct member names. A formal header cannot contain another group or a function-kind parameter: groups are flat abbreviations, not functions that construct interfaces.
A formal declaration has no header region parameters. Each member signature declares its own region parameters exactly as an ordinary function signature does, and each member call instantiates them under FORM-8 and the ordinary OWN rules. Actual-declaration regions instead capture store brands inside the formal's type arguments under FN-2; they are not member loan regions.
A formal member's parameter, result, region, effect, requirement, and ensures formation follows the same rules as an ordinary callable declaration; it has no body or trusted proof. Its contracts constrain admissible actuals [FN-4] and may be used only after those bindings have been checked.

A `pack_use` in a function or nominal generic parameter list names a formal declaration and writes one fresh type or const binder for each header parameter, in order. For example, `fn find<Key<K, E>>` declares the two written binders with Key's respective bounds, followed by one distinct function-kind parameter for each Key member. The same formal may be used more than once with distinct written applications.
A forwarding `pack_use` names those already declared binders and their function-kind parameters; it declares nothing. `Key<K, E>` in a call's or nominal's argument list forwards the complete expanded vector. No expected type or argument omission supplies a behavior.
Generated member identities are indexed by the receiving declaration, its written pack application, and the member ordinal. They are hygienic: they neither introduce an unqualified lexical name nor capture a local spelling such as `hash`.
The abbreviation names do not survive in type identity: two nominal applications with the same expanded type, const, function, and captured-brand vectors are the same instance, even when their actual group names differ.

An `actual` declaration names one ordered argument group for its explicitly written formal application. Its optional region parameters retain the exact store brands written in that application; every use supplies them explicitly. It declares no type, value, conformance, or implementation attached to a type.
For each formal member in source order it writes exactly one `fn_bind` in that order, with the matching left name and an explicitly selected right function. Missing, extra, repeated, unknown, or out-of-order members reject under FN-3 at the offending binding or the complete actual declaration for a missing member. No function is selected by name similarity, signature search, expected result, or a type-owned implementation.
All bindings are checked together at the declaration by FN-4; no partial group is published. A zero-member formal and matching empty actual are legal abbreviations.
`find::<SeedKey>` and `Map<'s, SeedKey>` expand the named actual's complete argument vector in their written position. A raw function-kind argument is `fn seed_hash`, or `fn helper::<T, n>` with every type, const, and function argument supplied. It must denote a fully instantiated ordinary function or an in-scope function-kind parameter, never a table operation, constructor, runtime expression, or partially applied generic function.
An actual group's member may be forwarded by its explicitly qualified name. Its dependency graph must be acyclic; a cycle is an FN-3 error naming that cycle, rather than an attempt to evaluate a group.

[FN-4] Every function-kind binding is checked against the instantiated formal signature before use. Named groups check all members at declaration; raw arguments receive the same check at their written argument.
Parameter and result counts, modes, and exact types must agree in order, with formal loan regions alpha-renamed by their ordinary signature positions [FORM-8]; parameter and result binder spellings are not signature identity. Captured store brands retain exact identity. An actual's region parameters fixed by those explicitly supplied branded types are substituted before the remaining universally quantified loan regions are compared. Corresponding unbound member regions have identical PROV-6 bounds, including the absence of a bound; a captured actual store region must satisfy its declared bound before substitution. An implementation cannot add a hidden region-domain restriction to a formal.
Each actual read, write, or allocation path must be covered by a formal path in that same category: after parameter-ordinal and field-ordinal normalization, the formal path must be a prefix of the actual path. Arena-allocation regions must agree after the same region substitution. The actual's own declaration must independently satisfy EFF-1 and exhibit exactly its own row under EFF-2.
Ordered requirements and ordered ensures, including their result selectors, must be structurally equal after type, const, function, parameter, result, and region substitution and expansion of local `define` bindings. This is structural matching of the written contract terms, not an implication solver, a logical-law assumption, or permission to reorder clauses. Each actual's requirements and ensures have the ordinary FN-8/FN-9 formation and verification boundary, including PRE-1 declarations.

A mismatch names the member or raw argument, the differing signature, row, contract, or result ordinal, and the restructuring `supply an explicitly matching function or change the formal interface`.
No reflexivity, transitivity, ordering consistency, hash/equality compatibility, or other algebraic law follows from a binding. Ownership, range proofs, initialization and cleanup must hold even for inconsistent supplied behavior.

[FN-5] There are no function values, methods, receivers, implicit Self, dictionaries, or dynamic dispatch. Closed-set runtime dispatch is `match`.
An unqualified IDENT call may name an in-scope raw function-kind parameter. `Key::hash(...)` names the corresponding member of the unique in-scope application of formal Key; when two applications of Key are present it is rejected at the callee and the source must write the full application, for example `Key<K1, E1>::hash(...)`. Selection uses distinct written applications before substitution, even if their eventual concrete types coincide. Two identically written applications require separately named raw function-kind parameters rather than a pack alias.
At a member call, named value arguments use the formal signature's parameter names in declared order, not the actual implementation's parameter names. The selected function's type, const, and function arguments were supplied explicitly at binding or group instantiation under FN-2; the already bound member accepts no further specialization. Its per-call regions follow FORM-8. Requirements, ensures, and effect attribution use the instantiated formal interface. Bare exclusive measures in ensures denote the exit state, and `entry(parameter)` denotes the entry state [MSR-3]; CALL-6 kills overlapping formal-row support before publishing the verified relations.
Every concrete function binding resolves to one ordinary function instance before IR. Lowering emits a direct call to that instance with the ordinary ABI, preserving its ordinary signature and ownership; group expansion emits no adapter call, storage, dispatch, dependency, or runtime proof check.
Bodies retain FN-2's symbolic spelling check and FN-2/FN-9's concrete-instance rechecks. A concrete instance uses formal contracts and the authoritative formal row at each bound call while independently checking the selected actual under the same signature, row, and contracts.

[FN-6] Recursion is permitted. Instantiation is finite by a structural rule over the finite written dependency graph, not by executing compile-time code or by a work, depth, or time budget.
The graph contains source function and source nominal templates; its edges include calls, instantiated nominal uses in signatures and fields, function arguments, and calls through function-kind parameters after their finite explicit bindings are resolved. Group abbreviations are expanded before the graph is checked. All edges and their complete type, const, and function argument vectors are retained.
Within every recursive component, each edge must forward the caller's complete parameter vector unchanged in position and kind, modulo alpha-renaming of region parameters. Constructing, specializing, dropping, adding, or permuting an argument on a cycle rejects under FN-6 at that dependency. This includes a growing nominal such as `Grow<T>` containing `box<Grow<box<T>>>`, and a call that wraps a function argument in a new specialized function on each traversal.
The diagnostic names the function/nominal cycle and the changed argument, with the restructuring `forward the complete generic argument vector unchanged on the cycle, or move the changing instantiation off the cycle`.
This criterion deliberately rejects some finite permutation cycles. Acyclic expansion is finite and a cycle creates no new instance key; deterministic checking visits each admitted instance. It assumes no behavior laws and uses no fuel.

[FN-7] Program start selects an ordinary function and supplies ordinary arguments [PROG-3]. Its name, signature, region parameters, result types, written contracts, and source callers obey FN-1 through FN-9 without an entry-specific restriction.
The compilation unit need not declare a function with any reserved entry name. Selection, argument construction and binding, and interpretation of a normal result belong to the build invocation and do not select source acceptance.

[FN-8] Every source `fn_decl`, generic or nongeneric, and every `fn_sig` may carry one optional `contract_block`. A function formal's block has the same formation rules and constrains bindings under FN-4. Every supplied definition must satisfy the ordinary declared contract; a Whitefoot body is checked under FN-9 and a PRE-1 declaration is supplied under SCOPE-3.
A present block must contain at least one `requires_clause` or `ensures_clause`; an empty or define-only block is an FN-8 rejection at `contract_block`.
Grammar fixes all definitions before all requirements and all requirements before all postconditions.

The definition scope initially contains the function parameters, named consts, and live type and const parameters, then each earlier definition after its complete initializer.
Every definition and clause expression must consist only of non-consuming datums and operation-table forms that are pure and total for every value in their selected operand domain.
Function calls, construction, move, borrow, subscript, mutation, control flow, allocation, and every proof-required exact or otherwise partial operation are inadmissible even when another clause states their domain, with one exception: exact addition, subtraction, and multiplication are admitted and are read as operations over the mathematical integers rather than as evaluations, exactly as an `affine_expr` is [INV-1]. A clause is erased before lowering and evaluates nothing, so a row whose meaning is total over the mathematical integers states a relation where it would otherwise request an operation, and no domain obligation arises to discharge. Exact division, remainder, negation, absolute value, and the shifts stay inadmissible: each has an input its own relation cannot state its way out of, so admitting it would place a partial operation where nothing discharges it.
The corresponding `.defined` queries are total and admissible.
Each definition produces an own copy value, follows ordinary typing and no-shadowing, and is erased by recursive alpha-expansion into every later clause; no definition is evaluated, snapshotted, lowered, or visible in the body.

Each requires expression is one `clause_expr` [GRAM-5, MSR-5], has exact mode and type `own Bool` under [OP-5], and independently forms one finite typed GoalTemplate after definition expansion.
A clause side's `+`, `-`, and `*` form that template's own operation nodes over the mathematical integers [MSR-5] and add no domain obligation, so a requirement side carries the whole affine expression and is not narrowed to the difference-bound fragment a published relation is [FN-9].
A formal datum keeps its zero-based parameter ordinal and field or `deref` projections; named consts, literals, selected operation rows, written arguments after substitution, result types, and operand order retain their existing identities.
Definition spelling, sharing, and NodePaths are absent after expansion.
The requirement occurrence is `(concrete function instance, requires_clause NodePath)` and is outside predicate equality.
Two predicates are equal only by exact typed-tree equality: there is no commutation, folding, reassociation, inversion, or De Morgan rewrite.
Signed decomposition, exact comparison-root L0 projection, and the fixed query-time Boolean introduction over independently proved children remain exactly [ENT-3, ENT-4, ENT-6].

At an ordinary source call, resolution, concrete instantiation, named arguments, exact types, borrow feasibility, and all actual-expression obligations complete first.
For every GoalTemplate in requires-clause source order, substitute each formal with that actual's Goal value identity in the same pre-transfer fact state: a borrow formal uses its resolved referent and an own actual its value before transfer.
A literal, named const, or place with field and `deref` projections remains an ordinary datum.
After every actual-expression obligation succeeds, an own actual whose complete
checked value belongs to [ENT-2]'s admitted exact-operation or index tree uses
that same structural Goal identity. If the complete value is outside that
admitted tree, it uses [ENT-2]'s occurrence-local call-argument
evaluated-value identity instead. No
exact operation or index identity is admitted before all of its nested domain
obligations succeed.
Every instantiated goal is judged independently in that unchanged state; a discharged clause adds no fact for a later clause.
The first refuted or unproved clause is the FN-8 call-site rejection and forms no checked program.
Only total success reaches ordinary transfer, effects, and normal return; no call receives a runtime fallback, alternate entry, or body clone.

At concrete body entry, every requirement goal is established independently as an [ENT-3] S4 source, in source order, with its own signed decomposition and exact L0 projection.
The clauses are never banded together.
There is no executable callee prologue, `llvm.assume`, optimizer license, or alternate lowering; later kills apply normally.
Direct and mutual recursion, forward calls, and every concrete generic instance use the same finite rule.

After all S4 sources and implicit parameter/type facts are closed under [ENT-4], a contradictory entry state makes that concrete instance legally uninhabited.
The checked body disposition is `Uninhabited { contradiction: DerivationId }`; it is success metadata, not a source rejection, and the derivation survives final identity remapping.
Syntax, resolution, type, ownership, effect, return-shape, statement reachability, call, and proof-form checks still inspect the complete source body, while proof obligations discharge under the contradictory state.
An uninhabited instance publishes no postcondition summary.
Lowering must preserve its ordinary ABI and symbol but emit exactly one empty entry block terminated by `unreachable`, without traversing or lowering any source statement.
A source call must still prove every contradictory requirement, which no reachable non-contradictory caller state can do.

[FN-9] Each `ensures_clause` in a FN-8 `contract_block` declares one independent normal-return relation.
A source declaration is not a trusted assertion: its body proves the relation by the selected-return judgment below. A PRE-1 signature supplies its declared relation under SCOPE-3 and has no source returns to check. Formation and caller instantiation are the same ordinary judgments in both cases.
No contract definition or clause contributes an effect, executable epilogue, runtime operation, storage slot, or runtime report.

Every declared result ordinal is a datum of every clause, written as that ordinal's `result_binding` spelling [CALL-4].
An unrouted clause is admitted only when every result ordinal it names is `own T` with T one [ENT-2] fragment integer after concrete [FN-2] substitution, or is `own T` with T a measured type [MSR-1] named under a measure former and nowhere else [CALL-4].
Its symbolic result datums are those ordinals' `result_binding`s.
A routed clause is admitted only as exact `when Ok(value: r):` or `when b is Ok(value: r):` for a result ordinal whose written type is `own Result<T,E>` with T a fragment integer, where `b` names that ordinal, r is that clause's fresh symbolic payload datum, and `Ok` and `value` retain their PRE-1 identities.
The ordinal binder may be omitted exactly when one declared ordinal has that enum type; two or more leave the route ambiguous and are refused at the declaration [CALL-4].
Route owner, ordinal, variant, field, and freshness admission precedes resolution of that clause expression [GRAM-10, TYPE-6].
The routed ordinal's whole-Result binder is unavailable in that clause; every other ordinal's binder remains a datum of it.
Borrow-mode, unit, float, aggregate, nested-payload, whole-Result, non-Ok, and every other shape remains a legal ordinary result but cannot supply a relation datum in this version.
Omitting Err routes means Err exits are unselected, not unreachable.

After recursively alpha-expanding every shared `contract_define`, the clause expression must have exact type `own Bool` and its root must be exactly one `compare_op` — `==`, `!=`, `<`, `<=`, `>`, or `>=` [GRAM-5].
Each operand is one **relation term**: one datum displaced by a written constant, which is the shape [ENT-4]'s closure represents and the shape every declared relation of the kernel declaration domain writes [BLK-0].
Its datum must be one of the clause's symbolic result datums, a parameter datum with field and `deref` projections, a named const, a typed integer literal, a measure of an admitted formal place P [MSR-5], or a measure of a declared result ordinal of measured type [CALL-4]; at least one operand contains a result datum (a measure over one included) or the exit-state measure of a `&uniq` parameter, and the two may name two different result ordinals. A clause naming only exclusive exit state is admitted regardless of the result type, including unit.
Its displacement is the mathematical value of the rest of that `affine_expr` side, which must reduce to one integer constant: the side is admitted exactly when it carries one such datum with coefficient one, or none and a constant, and a side carrying two datums or a datum with any other coefficient is outside the difference-bound fragment [ENT-4] and is an FN-9 rejection at that clause naming the fragment.
A measure rooted at a `&uniq` parameter denotes the selected return's exit state; `entry(parameter)` denotes that parameter at function entry [MSR-3].
No proof-required exact operation, computed arithmetic result, subscript, occurrence-local evaluated-value datum, Boolean connective, nested result projection, or body local becomes a relation datum; a clause side's own `+`, `-`, and `*` are the mathematical integer expression [MSR-5] fixes and are the displacement rather than an operation.
The comparison normalizes to one finite L0 RelationTemplate whose two terms carry their displacements as one folded constant; equality's two bounds remain one relation occurrence.
Parameter datums denote function-entry images, except that a bare measure rooted at a `&uniq` parameter denotes exit state [MSR-3].
The template retains parameter ordinals and projections, result ordinals, route declarations, named-const identity, literals, substitutions, comparison row, operand order, and normalized relation, while excluding result/route/definition spellings, definition sharing, and callee identity.
Its occurrence is `(concrete function instance, ensures_clause NodePath)`.

An unrouted clause selects every explicit return.
A routed Ok clause selects exactly a return whose routed ordinal's expression is a direct canonical `Ok<T,E>(value: atom)` and uses that payload atom as that ordinal's result datum; a direct Err there and propagated error exits are unselected.
Every other Result shape at that ordinal in a function with an Ok clause is an FN-9 rejection rather than an inferred route.
At a selected return, each result datum the clause names evaluates to one [ENT-2] term or constant, read from its own ordinal's returned expression; an ordinal the clause does not name imposes nothing.
For an ordinary inhabited instance, each clause's selected-return set is independently nonempty; an empty set rejects at that `ensures_clause`.
An [FN-8] uninhabited instance still checks route, type, expression, and return-shape source judgments, but is exempt from nonempty and proof requirements and publishes no relation.

A referenced `own` or shared-borrow parameter's measure, or a measure explicitly rooted at `entry(parameter)`, is that parameter's entry datum [MSR-3], which is minted at body entry, contains no place, and is therefore killed by nothing. A bare `&uniq` measure instead evaluates over that parameter's resolved referent immediately before each selected return, after the return's ordinary effects and kills; replacement of that referent changes this exit term and never retargets the entry datum. Non-measure parameter datums retain the entry-image stability rule below.
Every other referenced parameter entry image creates no snapshot term.
Its stability begins live at body entry and becomes permanently unavailable on the first structural edge whose [ENT-5] kill overlaps the datum, a holder used by it, or its support; join is intersection and contradiction never restores it.
An element write does not invalidate such an image, while a write to the place's own descriptor storage or to any prefix of it, or killing its root or holder, does [MSR-2]; at a call, which of the two a projected callee write is, is [CALL-1] through [CALL-3]'s classification and never the argument's shape [CALL-5].

For each clause in source order and then each selected return in NodePath order, the checker first completes ordinary return typing, obligations, calls, effects, and pre-return kills.
If an entry image is unavailable, the relation is unproved; otherwise substitute the result datum and query immediately before return transfer and edge cleanup.
The relation is queried once in the current ProofContext at that return.
Every query must discharge; the first clause/return failure rejects with no runtime fallback.

Postcondition verification has no summary fixed point.
Form the concrete ordinary-call graph, its SCCs, and the callee-before-caller condensation. PRE-1 supplied declarations are leaves whose declared relations are already available; source definitions undergo the following body verification.
While verifying a component, all same-component S12 summaries are unavailable; previously completed callee components remain available.
Only after every relation of every inhabited instance in the component succeeds are all its relation summaries published atomically; any failure publishes none.
Uninhabited instances contribute no summary.
Declaration or worklist order and iteration cannot change the result.

For one ordinary call c, `A0(c)` means resolution, concrete instantiation, named arguments, exact types, borrow feasibility, every actual-expression obligation, exact formal substitution, and success of every FN-8 requirement have all occurred in that order at the same pre-transfer point.
Failure forms no postcondition candidate.
For one relation q, `M(c,q)` holds only when q's route matches that exact establishment event, result and referenced formals substitute independently to live [ENT-2] terms or constants after ordinary kills, and no referenced actual is represented only by an occurrence-local evaluated-value datum.
A discarded or nested result, stored or propagated whole outcome, unsupported or unselected route, killed support, or nonterm actual makes only the relations that reference that unavailable datum false under M. A relation naming no result needs no result destination and establishes on an ordinary successful call continuation, including a unit-returning or discarded-result call.

Subject to A0 and M, failure-atomic scratch establishes q after transfer, consumes, borrow commits, callee-effect kills, and target kills.
Every establishment retains its ordinary declared-relation parent, including the selected-return proofs for a source definition, plus all actual-obligation and requirement parents from A0.

All matching verified relations are established together on the admitted result route.
An unrouted fragment result establishes onto the fresh binding of a direct ordinary-let call and onto the target place of a direct ordinary `set` whose right-hand side is that call: one destination rule at two placements, the `set` placement established after that statement's own commit and target kills [CALL-6], where a substitution whose support those kills remove makes only that relation unavailable.
An unrouted relation over a declaration's result ordinals establishes onto the binders of a destructuring `let` and onto the targets of a `set` target list, ordinal i onto binder or target i [GRAM-4, CALL-4]; an ordinal whose destination is no [ENT-2] place makes only the relations naming it unavailable.
A selected Ok payload establishes only when the ordinary call is the direct scrutinee of a `match_stmt` or `value_match`, at entry to its exact direct `Ok(value: payload)` arm.
A named, stored, aliased, propagated, discarded, or otherwise indirect whole outcome carries no pending summary token.

The existing narrow receiver routes remain per relation.
For `set x = user_call(...)`, x is a live bare own fragment of the exact result type and exactly one argument is direct non-consuming x; after transfer, effects, commit, and kill, a relation may substitute result with post-write x only when it omits the formal supplied by x and all other supports remain live and disjoint [OWN-7].
For a selected payload, the first arm statement may be exactly `set outer = payload;`; after the ordinary commit and kill, replace only result-payload occurrences in each established relation with post-write outer when every other support remains live.
These routes establish no equality and every projected, consuming, repeated, aliased, nonfirst, wrong-type, wrong-binder, or unsupported form establishes nothing.

All candidate S12 and delivery facts remain in one failure-atomic scratch batch until the current statement's ordinary transfer, effects, ownership commits, and kills succeed; then the whole batch commits once.
No candidate is individually committed or retracted and no second flow walk or negative fixed point exists.

Every successful selected-return proof and caller establishment extends [DIAG-2]'s one derivation DAG.
Postconditions add no runtime operation, hidden check, assume, optimizer license, alternate lowering path, or ABI field.

[MSR-5] A contract clause is the relation an invariant already is, over a wider operand set.
A `requires_clause` and an `ensures_clause` take a `clause_expr` [GRAM-2, GRAM-5]: one `affine_expr`, or two around one `clause_op`, where an `affine_factor` is an `atom`, a `call`, or a constructor `call` [GRAM-4].
The operand set is the whole of this rule: [GRAM-5]'s `atom` has no `call` alternative, so before this version a measure of a place derived nowhere in a clause and `len_of(source) <= len_of(out)` was a GRAM-5 parse rejection at the comparison, while the same fact written through one `contract_define` per operand was admitted — one semantics with two spellings, one of which cost a definition per measure.
A clause side is that same `affine_expr`, so `ensures len_of(rest) <= len_of(vector) + 1_u64;` states in one clause the relation a `header_invariant` states in one invariant, and the two placements of one relation share one production rather than one spelling each.
The `+`, `-`, and `*` of a clause side denote the mathematical integer expression [INV-1] fixes and perform no [OP-1] operation, so a clause side creates no [OP-2] domain obligation and admits no runtime value it could overflow; a `clause_op` is exactly the Bool-valued rows of [OP-1], the six comparisons and the five infix `defined` queries, and the arithmetic operators are consumed inside the side.
A clause is judged by exactly the [OP-5] condition [FN-8] and [FN-9] already apply: the root has exact value mode and type `own Bool`, and every operand is a non-consuming datum or an operation-table form pure and total over its selected operand domain.
This rule adds no route, no fact source, and no proof authority; it adds spellings the existing admissions already accept.

The measure formers are table data over the measured types, with four rows in this version — `len_of(P)`, `cap_of(P)`, `room_of(P)`, and `head_of(P)` [MSR-1], each of fragment type u64 and each admitted for exactly the places [ENT-2] clause (b) admits a measure term for.
A measure former is written as an ordinary `call` whose one `atom_list` operand is that place; a written type argument, a `fieldinit_list`, or a second operand is the ordinary [OP-1] rejection.
A clause operand is also an in-scope const generic [MSR-6], which is a constant term and not a measure former.
A clause operand that is neither an [ENT-2] term nor a constant stays an ordinary pure total operand contributing no L0 projection; clause position makes nothing a term.

The written affine surface carries the same four formers.
The `affine_factor` production of [GRAM-4] admits an `atom`, a `call`, and a constructor `call`, and [INV-1] admits there exactly the four measure formers over an admitted measure place and the bare integer literal and live own integer local it has always admitted, so a `header_invariant`, an `invariant_stmt`, and a `proof_use` state a relation over a measure in the spelling this rule already gives a clause.
One production carries both placements and each rule states its own admitted factors; a factor [INV-1] does not admit is a hard error citing INV-1 at that factor and never a parse rejection.
The affine domain of [ENT-6] is unchanged by that: it carries one compiler-owned atom per live measure term either way, so what the widened production adds is the writer's ability to *state* the relation, never a new derivation.
Before this version the production was not widened, so `invariant grown: len_of(built) >= at` was a GRAM-4 rejection at the former and the invariant placement of this rule's own relation admitted no measure at all.

[CALL-4] Contract vocabulary, the result ordinal, the routes, and where the relations land.
The clause operands of [FN-9] are terms [MSR-5], so a measure over an admitted formal place is an operand with no per-family admission, and so is one over an admitted result place.
A `fn_decl` declares one result or an ordered result list of two or more [GRAM-2, FN-1], and each `result_binding` is one **result ordinal**, numbered from zero in written order.
Every ordinal is a datum of every clause, written as that ordinal's binder spelling, and a single-result declaration is the one-ordinal case of this sentence rather than a second rule.
A result ordinal's declared type is a fragment integer after concrete [FN-2] substitution [FN-9] or a measured type [MSR-1], and which of the two decides what that ordinal supplies: a fragment ordinal is a datum of the clause as its own value, and a measured ordinal is a datum only under a measure former, whose operand is that ordinal's place.
A measure over a result place is instantiated at that ordinal's own destination [ENT-3.S12] — the place the destination names — exactly as a measure over a formal place is instantiated at the formal's, and is queried at a selected return over the place that return hands back.
DEFERRED: a measure over a result place formed with field-selection `psuffix`es, `deref` wrappings, or subscripts, in place of the bare result place this version admits; its delta is numbered rules +0 and grammar productions +0, and it is an admission widening of [FN-9] rather than a new judgment.

A routed clause is written `when V(f: r):` or `when b is V(f: r):`, where `b` names the result ordinal the route applies to.
The ordinal binder may be omitted exactly when one declared ordinal has that route's enum type; when two or more do, the route is ambiguous and the declaration is a hard error citing CALL-4 at the `ensures_clause`, `AmbiguousResultRoute`, carrying the restructuring `name the result ordinal the route applies to: write `when b is V(f: r):``.
The judgment is at the declaration because the ordinal set is fixed there, exactly as [CALL-6]'s consistency judgment is: a route no reader can attribute to one ordinal publishes a fact about a value the writer did not name.
DEFERRED: a route over any variant of any returned enum type, in place of the prelude `Ok` this version admits; its delta is numbered rules +0 and grammar productions +0, and it is an admission widening of [FN-9] rather than a new judgment.
It is deferred with the measured result above, because the route identity a clause retains is a prelude declaration ordinal and generalizing it is one change with that widening rather than two.

The destinations are exactly [ENT-3.S12]'s closed list, and a relation reaches a caller only there; [CALL-6] fixes the point at which each is instantiated and the point at which each is established.
That list gains the two destinations a multi-result contract creates, and only a multi-result contract exercises them: **each binder of a destructuring `let`** and **each target of a `set` target list** is the S12 destination for every published relation naming the value that lands there, ordinal i landing at binder or target i [GRAM-4].
Both are established as [CALL-6] states, on the call's normal continuation and after that statement's own commits and kills, so a target list's relations are established over the values its commits left and never over the ones they replaced.
Neither the arm binder of an own-place `match` whose scrutinee is not the call itself nor the destructuring-consume binder is a destination of its own: each needs a relation to survive a naming event between the call and its destination, and [MSR-3]'s placement table is what carries one across such an event.
A published [MSR-1] measure of the transferred value reaches both, because the payload and destructuring placements carry exactly that; a measure datum carries nothing else.
DEFERRED: the same two positions for a published relation that is not one of [MSR-1]'s measures of the transferred value, which names terms no placement mints. Its delta is numbered rules +0 and grammar productions +0.

## 9. Effects (unified-state revision)

[EFF-1] Row grammar: the `effects` and `effect` productions of the fence below, in exactly this canonical order (reads, writes, allocates).

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" effect_path ("," effect_path)* ")"
        | "writes" "(" effect_path ("," effect_path)* ")"
        | "allocates" "(" ( effect_path ("," effect_path)* | ("arena" REGIONID)+ ) ")"
effect_path := IDENT ("." IDENT)*
```

A category appears at most once in one row.
`pure` is the unique spelling of the empty row.
Frame residency (STOR-1) is not an allocation by definition.
The spellings `external`, `blocks`, `memory`, `world`, and `capability` are not grammar atoms, effects, retired spellings, or reserved words. They satisfy IDENT wherever any other lowercase identifier does.

Every `effect_path` is rooted at one formal value parameter of the same callable. Each suffix selects one statically known ordinary struct field from the preceding type. A root resolving to a local, result binder, unrelated declaration, or non-parameter declaration is an EFF-1 rejection. An unknown field, an enum payload, a dynamic subscript, a dereference spelling, and every other place form are outside this candidate grammar. A bare parameter names the complete state that parameter supplies; a field path names only that structural substate.

For a borrow parameter, its effect path names the borrowed referent rather than the local reference representation. For a direct `Slice<'r, T>` parameter, it names the viewed backing state rather than the descriptor. For an `own` parameter, the path names that parameter's current storage. Merely moving, returning, or structurally repacking that value does not observe or change it; an operation which reads or changes its contents exhibits the corresponding path. A REGIONID never names effect identity: regions state loan liveness and outlives relations only.

The row describes observations and changes of ordinary Whitefoot state and allocation. It does not distinguish memory from outside state and does not describe a host scheduling mechanism. Opaque nominals, buffers, aggregates, and providers all use the same path, exactness, call-substitution, and ownership rules. No type or path carries a writer-visible capability category.
`reads(path)` means the operation observes that state. `writes(path)` means the operation replaces or advances that state. They remain independent exact facts: an operation which observes prior state while changing it names the path in both categories, while a complete overwrite need only write it.
`allocates(path)` [S23] means the operation acquires storage from the store whose provider that path selects, and the entry takes the same formal-rooted paths the other two categories take: an `allocates` entry is exhibited exactly when the body reaches an allocation whose provider argument projects to that path under [EFF-2]'s call-boundary projection.
An allocating row names the same provider path in all three categories, in this rule's canonical order `reads(p), writes(p), allocates(p)`, an allocator observing its prior state while changing it; a release exhibits `writes` of the resolved provider place and no `allocates`, spending the store's capability without acquiring from it [PROV-6].
A function *reaches a store* when its own row carries an `allocates` or `writes` entry whose selected type at the leaf is that store's provider type, or when it calls one that does; the unit being closed [PROG-1], that transitive closure is exact and is computed from signatures alone.
Storage whose store's provider is not a value has no `effect_path`, and the two such stores this version still carries are treated differently because they differ in what a caller must know.
The ambient heap of `box<T>` and `buffer<T>` [STOR-1] has no written entry at all: it is reached from every scope, its release resolves no provider place and carries the empty row [PROV-6], so an allocation of it is checked, reachable and reclaimed exactly as before while contributing nothing writable to a row.
The region-bounded storage of `arena<'r, T>` [STOR-2] keeps the entry `allocates(arena 'r)`, whose REGIONID is a region position of the declaration [GRAM-2]: an allocation into a *caller-supplied* region is a write of the caller's own storage, and dropping it would leave a callee's arena allocation invisible at a call, which is [PAR-1]'s overlap footprint. That alternative of the production is transitional and retires with `arena<'r, T>` and its `arena_new` row; its retirement delta is grammar productions +0 and unique fixed lowercase grammar atoms -1.
Neither absence is a licence: allocation admission still uses the retained allocation record and its required domain proof rather than absence of a written row.

[EFF-2] A concrete function declaration exhibits the union of its resolved body accesses, calls and allocations, and its ordinary memory-reclamation effects.
The body contribution is syntactic over the complete function body. Erased definitions and contracts [FN-8, FN-9], proofs, and the compiler-owned captures, comparison and update of a counted loop contribute nothing.
Every source occurrence contributes even in an FN-8 uninhabited instance. No path condition, constant evaluation, proof, optimizer result, or backend reachability narrows the conservative structural normal-control graph [FN-1].

At a function-kind parameter or named member call [FN-5], the instantiated formal's written row is the authoritative callable boundary. Its paths undergo the same resolved-place projection and ENT-5 support kill as ordinary callee paths; the selected actual's narrower covered row never substitutes for that boundary. Generic spelling checking retains formal-rooted contributions under the written bounds, including an owned parameter that later has a copy instance. Concrete rechecking verifies the fixed containing declaration's row under the same formal interface and independently checks each actual against FN-4 and its own row. No adapter, dummy read, widened runtime access, or law supplies an exhibited path.

Every read or write is attributed after ordinary place and holder resolution and [OWN-5] view provenance. An access rooted in a formal contributes the most precise static struct path EFF-1 admits for that resolved place; a dynamic element or range maps to its nearest statically nameable enclosing path. A borrowed referent is the holder's resolved place, and a direct view projects through its complete origin set. A multi-origin view contributes the deduplicated union of its formal-rooted origins.
An owned parameter's place is that parameter's current storage, not a value-history identity. Moving, returning, or structurally repacking an owner contributes no read or write merely by transferring it. After transfer into a local binding or aggregate field, subsequent accesses are attributed to that destination's resolved storage. A whole-place replacement changes the value at that place without changing the place's effect root. No effect root follows an owned value through moves, results, or replacements.
A const root and `immutable-const` contribute no read effect. An access rooted only in local storage contributes no enclosing formal-rooted effect, including through a local borrow or view; the checked access and its ordinary footprint remain.
A write is admitted only when ordinary ownership already grants exclusive or owned access. An effect path grants no permission, changes no loan extent, and cannot narrow a borrow of a whole aggregate to one field.

At a call, each declared effect path selects its root formal's actual argument and appends its static field suffix to the actual's resolved place. Holder resolution reaches the borrowed referent; a view actual projects through its complete origin set. The resulting ordinary read or write footprint is retained at the call. A projection rooted in a current formal contributes that formal's corresponding path; a projection rooted only in local storage contributes no enclosing path. Equal lifetime arguments never merge distinct suppliers.
The actuals' call data are captured after argument evaluation and before the call. Each projected write kills overlapping support under ENT-5, then CALL-6 instantiates entry terms from that call datum and exit terms from the actual's resolved return place. This substitution does not inspect the callee body and retains no owned-result or exclusive-referent ancestry.
Framing an action out of an enclosing row removes no checked action or ordinary effect footprint. Optimization is governed by EFF-3 and the source's value, ownership and control semantics.

The ordinary memory-reclamation contribution is the union of `writes` on the provider places resolved by PROV-6 for every reclamation that may run on a conservative normal-control edge. A branch-specific reclamation contributes on the same terms as an unconditional one. Each owner has exactly one disposition per edge: transfer, explicit consumption, or the one admitted compiler-derived drop; linearity may refuse that drop [PROV-6, STOR-3]. No opaque nominal contributes a drop action or a type-specific effect.
Reclamation that spends no provider capability contributes nothing. A `dispose p;` is a written body occurrence: it contributes its ordinary write on p's ultimate storage origin and each resolved provider place [PROV-6, LIV-2].
A SET-1 commit contributes a write; a SET-2 exchange contributes a read and a write of the same resolved target. LIV-2's reinitialization of a complete binding already dead at statement entry keeps its no-previous-owner exception. Target and right-hand-side evaluation contribute ordinarily.
Rows are checked both ways against this complete exhibited set: undeclared-but-exhibited and declared-but-unexhibited are both EFF-2 errors. A mismatch contributed solely by memory reclamation uses the function's complete `effects` node and identifies the missing or extra provider path. A declaration with no exhibited contribution writes `pure`, whether or not it carries erased contracts.
A PRE-1 function signature is the ordinary declared boundary; its linked definition must satisfy the same boundary [SCOPE-3, PRE-1]. No source body is fabricated for it and no alternate effect rule applies to its calls.

[EFF-3] A call whose row is `pure` licenses deduplication and reordering with equal arguments.
Elimination of an unused such call additionally requires a termination proof; v0 provides no termination checker, so unused calls are not eliminated.
The source spelling `pure` excludes state reads, state writes, and allocations; it does not promise termination.
A call that exhibits `writes(path)` may remain observable even when its result is unused. A call on fresh local state retains that instantiated effect even though it frames out of the enclosing signature. No optimization may erase, duplicate, speculate, or reorder either call unless ordinary effect-path overlap, closed-state, escape, ownership, control, result, release, and surviving-observer proofs establish the exact transformation.

[EFF-4] Accepted source has no writer-reachable abort effect, exception, unwinding edge, or hidden runtime proof fallback.
Every proof failure rejects the source before lowering.
Unavailable resources and a trusted-computing-base failure remain outside the source effect system under [SCOPE-3]; none creates a writer-visible effect spelling or an alternate successful source judgment.

## 10. Errors

[ERR-1] Recoverable errors are values: prelude `Result<T, E>` and `Option<T>` (§14), dispatched by `match`.
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

[ERR-4] Classification: expected environment and input failures represented by an operation contract are values (`Result`); unproved function, operation-domain, allocation-fit, bounds, layout, address, and target-domain obligations attached to source execution are source rejections.
Unavailable external resources and trusted-computing-base failures remain outside the source outcome model under [SCOPE-3].
An operation's classification is fixed by its table row and attached static obligations, never by call-site preference.
The overlap permissions of [PAR-1, PAR-2] are implementation permissions over an already accepted sequential program, not source obligations in this version: absence of a complete permission derivation retains sequential lowering and never rejects the source.
If an implementation does select an overlapping lowering, every premise of that permission must be discharged before emission; a failed premise cannot be repaired by a runtime check or partially parallel fallback.

## 11. Programs, closed world

[PROG-1] One closed compilation unit is formed by PROG-2. Every language name is defined within it, by the prelude [PRE-1], or by the kernel declaration domain [BLK-0].
This version has no source include, import, module, source-path lookup, or separate-compilation form.
Build and link supply definitions for ordinary declarations and select the invocation [PROG-3]; implementation language and linkage are not source semantic inputs.

[PROG-2] One compilation unit is one ordered nonempty sequence of logical source records.
Each record contains one logical path and one exact source-byte sequence.
A logical path is an ASCII relative path made from one or more nonempty components separated by exactly one `/` byte, with no leading, trailing, or repeated `/`; each component contains only ASCII letters, ASCII digits, `.`, `_`, or `-`, and no component is `.` or `..`.
Path spelling is preserved exactly and compared case-sensitively.
An empty record sequence, an invalid logical path, or two records with the same logical path is an input-envelope failure, not a source-language rejection.
Record order is exactly the order in the bound invocation; no path sort, host enumeration order, or other reordering is applied.
Within that bound unit, a source record is identified by its zero-based ordinal, exact logical path, and exact source bytes.

[PROG-3] Execution starts by an ordinary call to the build-selected function with arguments matching its ordinary signature. The implementation must establish the arguments' declared types, ownership and region validity, and requirements before making that call, exactly as any caller must [FN-1, FN-8].
A program may use the ordinary PRE-1 `Inputs` struct, pass a `Heap<'s>` as a separate ordinary parameter, or use any other admitted signature. A provider cannot be hidden inside an aggregate where STOR-5 refuses it. No region is implicitly minted in source because a function is selected at start.
The ordinary call ABI, loan duration through return, result transfer, and scope-exit rules apply to both Whitefoot and linked definitions. Implementation engines may wait or schedule internally only while preserving this same boundary. The build interprets returned values and performs any invocation teardown outside the source call; neither operation adds a source effect or changes acceptance.
Resource unavailability before that call and trusted-computing-base termination remain outside the source outcome guarantee [SCOPE-3].

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
If the boundary token is one member of four consecutive actual tokens `IDENT "." IDENT ("("|"::")`, that dotted call-or-targs spelling cites [FORM-3].
Its coordinate is the complete interval from the first IDENT through the second IDENT.
An allowed suffix would already be one maximal OPNAME token, while a field place cannot be called or given targs.
This bounded diagnostic window may include already recognized tokens, performs no operation-table or name lookup, consumes nothing, and does not enlarge recognition's two-token lookahead.
2.
If source-EBNF provenance reaches or would next enter an `atom` occurrence in `atom_list`, `fieldinit`, an `infix` operand, the subscript offset, or either endpoint of a `for_stmt`, and the two actual tokens at the start of that occurrence are `(IDENT, "(")`, `(IDENT, "::")`, `(OPNAME, "(")`, `(OPNAME, "::")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the rejection cites [GRAM-9]; in an infix-operand occurrence, a two-token start whose second token is an `infix_op` or `compare_op` token — the forbidden nested-infix start — likewise cites [GRAM-9].
These are exactly the `call` and constructor `call` starts forbidden in an atom-only position; no name lookup participates.
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
Thus traversal reaches a direct name, a name inside a consuming list arm, or a name after one or more skipped nullable prefixes such as `doc?`, but cannot tunnel through structural choices such as `item`, `stmt`, `expr`, `atom`, `callee`, `pbase`, `targ`, `contract_define`, `requires_clause`, `ensures_clause`, `result_route`, or `atom_list | fieldinit_list`.
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

An input-envelope failure, resource failure, target-layout failure [STOR-6], compiler-invariant failure, unsupported compiler capability, backend failure, or external-tool failure is not a source-language rejection, cites no language rule, and carries no expected-terminal set.

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
A generic numeric literal carries its lexical generic-type use on the suffix; its numeric value remains the literal judgment, not a second deferred role.
A struct TYPEID remains one declaration event producing two domain entries, not two events.

Within one owner node, distinct direct grammar-role carriers are ordered left to right by their complete carrier coordinates; distinct carriers with identical complete coordinates use the closed class order declaration, result-reservation, lexical-use, deferred-use.
The zero-based carrier index is `role_ordinal`.
`subtoken_ordinal` is zero for a role covering its complete carrier; embedded semantic name roles are numbered from one in byte order.
No current production gives one carrier both a complete-token role and an embedded role; the generic-numeric suffix retains subtoken ordinal one.
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
2. an OWN-3 repeated REGIONID declaration within one function declaration or function-formal signature, parameters included;
3. a GRAM-10 match-binder freshness violation;
4. a declaration collision with PRE-1;
5. a compilation-root duplicate or same-lexical-scope redeclaration; and
6. a nested declaration shadowing a live declaration.

Each declaration or result-reservation event forms an inventory candidate only for an applicable rank above; an event for which no rank applies forms no candidate.
The stage selects the minimum canonical event key among events with at least one candidate and then the first applicable rank at that event.
A FORM-3 reservation payload is `(spelling, carrier_role, reserved_class, inventory_ordinal)`.
Its `spelling` is the complete declaration or result-candidate spelling.
A REGIONID payload uses its unsigiled IDENT-shaped interior while the rejection coordinate retains the complete sigiled token.
Its closed carrier roles are function, named-const, parameter, contract-definition, let, for-binder, match-binder, result-binding, route-result, field, variant-field, region-parameter, local-region, and invariant.
`reserved_class` is dotless-operation or mode-word.
A dotless-operation ordinal is the zero-based first occurrence among distinct operation-family spellings, scanning OP-1 rows top to bottom and each `op` cell left to right and skipping every later occurrence of the same spelling; both `cvt` rows therefore name one family and one ordinal.
A mode-word ordinal is the zero-based FORM-3 alternative order `wrap`, `defined`, `checked`, `sat`, `strict`.
Those two reserved sets are disjoint in this version.
An OWN-3 repeated-region payload is `(spelling, conflicting_region_origin)` and points to the later region declaration; OWN-3 precedes GRAM-10 in the rank even though no grammar carrier can be both a region declaration and a match binder.
For the GRAM-10 violation defined by TYPE-6, the payload is `(binder_spelling, paired_field_spelling, optional_earlier_binder_origin, ordered_arm_entry_live_lexical_ident_origins)`.
Earlier binders and arm-entry origins are ordered by declaration-event key.
That binder does not also create a TYPE-6 duplicate or shadow candidate.

A declaration collision payload is `(spelling, ordered_nonempty_conflicts)`; it cites INV-1 when the later declaration is an invariant name and TYPE-6 for every other declaration domain.
Conflict domains use the fixed order lexical-IDENT, nominal-type, constructor, numeric-bound, REGIONID, LABEL, invariant.
Each conflict contains its domain, declaration class, and `conflicting_origin`; conflicts within one domain use PRE-1 declaration ordinal first, then source declaration-event key.
A source origin is `(NodePath, SourceCoordinate, role_ordinal, subtoken_ordinal)`; a PRE-1 origin is `(PRE-1, declaration_ordinal)`, with the zero-based preorder fixed by PRE-1. A kernel origin is `(Kernel, container_declaration_ordinal)`, with the zero-based preorder fixed by BLK-0. Prelude and kernel declarations are admitted to every compilation unit [PROG-1].
A struct event may report both nominal-type and constructor conflicts in that order.
Rank 4 reports only PRE-1 conflicts when the same event also conflicts with source.
A PRE-1 collision points to the source declaration.
Rank 5 points to the later source declaration event.
Rank 6 points to the nested declaration, including one shadowing a source-later but whole-unit-visible function.
Every declaration-inventory rejection uses `SourceNode` at the declaration role and has no expected-terminal set.
An FN-9 result-datum reservation instead uses `SourceNode` at the owning `result_binding` or `fieldbind`, a coordinate equal to the candidate IDENT token, and the FORM-3 payload above; it creates no TYPE-6 runtime declaration or duplicate event.

If inventory succeeds, every lexical use admitted by TYPE-6, OP-1, INV-1, or PRF-1 creates one lexical-use event.
The generic-numeric suffix admits a live generic TYPEID parameter; FN-3 and FORM-5, not lexical resolution, later require its numeric bound.
Lexical resolution fixes only the declaration or operation-family target.

The closed declaration-class order is function, function-parameter, named-const, const-generic, value, generic-type, nominal-type, struct-constructor, enum-variant, numeric-bound, formal, actual, region, label, invariant, operation-family.
TYPE-6, OP-1, INV-1, and PRF-1 fix each lexical role's ordered admissible subset.
A use's exact-spelling candidate universe contains all compilation-root entries in its grammar-selected domain and, for non-root declarations, only entries belonging to its declaration-owner chain.
All sibling or expired lexical scopes within the same `fn_decl` owner participate so that an out-of-scope same-function declaration can be distinguished from absence.
A function-formal signature admits declarations of that signature and its enclosing declaration ancestry but not declarations owned only by a sibling member signature.
A struct, enum, formal, or function generic belongs only to that declaration and its descendants.
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
| numeric-bound TYPEID or the group TYPEID of `pack_use` | FN-3 |
| constructor `call` constructor TYPEID, enum-variant-only `arm` TYPEID, or `result_route` TYPEID | TYPE-6 |
| REGIONID use | OWN-3 |
| LABEL use | TYPE-6 |
| `const` IDENT | CONST-1 |
| `cvalue` IDENT | CONST-2 |
| `pbase` IDENT | TYPE-5 |
| IDENT or OPNAME `callee` | OP-1 |
| unqualified `fn_bind` or `function_arg` callee IDENT | FN-3 |
| FORM-5 generic-numeric TYPEID suffix | FORM-5 |
| affine IDENT in a `header_invariant` or `invariant_stmt` target | INV-1 |
| affine IDENT in a relation-form `use_premise` | PRF-1 |
| the named multiplicity IDENT of `proof_use` | PRF-1 |
| IDENT premise of `use_premise`, an invariant name | INV-1 |

A successful non-LABEL lookup has exactly one visible admissible target; a successful LABEL lookup has exactly one enclosing target.
A rank-1 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_nonempty_invisible_origins)`.
A rank-2 payload is `(spelling, lexical_use_role, ordered_nonempty_label_origins)`.
A rank-3 payload is `(spelling, lexical_use_role, ordered_admissible_classes, ordered_available_classes)`, where available classes are visible exact-spelling entries in that use's candidate universe, listed once in the closed class order and possibly empty.
Complete IDENT, TYPEID, OPNAME, REGIONID, and LABEL use spellings include any sigil; only the generic-numeric suffix spelling is bare `T`.
This is declaration-kind resolution, not type checking.
Across use events the minimum event key wins.
Every resolution rejection uses `SourceNode` at the use role and has no expected-terminal set.

The dependent-declaration carriers are exactly the `field` and `vfield` declarations. A `fn_sig` instead contributes a function-parameter declaration with the scope and qualified-member disposition stated by TYPE-6 and FN-3.
Each is a declaration-class carrier that produces one dependent-declaration record and one declaration event for later typed owner/member checking, but none enters a resolver lookup inventory.
The two field carriers participate in FORM-3's reservation inventory; a function-parameter name has the ordinary function-name reservation judgment.
The deferred-use carriers are the left IDENT of `fn_bind`, the member IDENT of a qualified `callee`, the first IDENT of an arm `fieldbind`, each `fieldinit` IDENT, each `psuffix` IDENT, and each field selected below an effect-path root.
Each produces one deferred-use record for later typed owner/member checking.
The `result_binding` IDENT and the second IDENT of a `result_route` fieldbind are FN-9-owned result-datum carriers.
The header candidate produces one reservation event with `role_ordinal` zero in its `result_binding`.
The route candidate produces one reservation event with `role_ordinal` one in its `fieldbind`, after the first IDENT's FN-9 field-owner carrier.
A result-datum reservation uses the canonical key above but is not a runtime declaration, lexical use, dependent declaration, or deferred use.
Candidates participate in FORM-3 reservation checking but provide no pbase target before FN-9 admission; they enter no owner/member lookup and no TYPE-6 duplicate or shadow inventory.
The header candidate is available only to an admitted unrouted ensures clause; after the leading route TYPEID's ordinary constructor lookup, FN-9 admits the route payload candidate only within that one routed clause.
No result datum is visible in a contract definition, requirement, function body, or different ensures clause.
The name of every `header_invariant` and `invariant_stmt` produces one proof-only invariant declaration record that uses TYPE-6's inventory and scope machinery; [INV-1] owns collision and lookup failure in this domain.
An IDENT premise of `use_premise` produces one lexical-use record querying only that domain; it can never resolve to a value declaration that happens to have the same spelling. A `proof_use`'s own IDENT is the named multiplicity and queries the value domain instead, so the two positions never compete for one spelling.
These records have no runtime declaration or value identity, but they participate in FORM-3 reservation and deterministic lexical resolution exactly at their stated scopes.
The retired law-name and law-argument roles produce no records. A function-formal expansion retains its written declaration and application identities rather than fabricating a second lexical spelling.
In an `arm` or `result_route`, the leading TYPEID first resolves globally to an enum variant.
Later typed checking compares that variant's owner with the scrutinee enum for an arm; a foreign arm variant cites TYPE-6.
FN-9 separately requires the route's successfully resolved variant and owner to be exactly PRE-1 `Result.Ok`.
The resolver does not otherwise accept or reject a dependent role's owner/member relation.

A missing whole-unit requirement is not fabricated as an inventory or lookup event.
Missing or duplicate formal members, field labels, and actual bindings remain typed-dependent rejections under FN-3 and the ordinary field-owner rules.

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

After complete lexical resolution succeeds, FN-3 validates the complete source-ordered formal and actual tables before FN-4 publishes any binding.
A repeated formal member rejects at the later `fn_sig` and its complete extent.
A malformed formal application, wrong-kind argument, or actual expansion cycle rejects under FN-3 at the application or binding that supplies the failed premise; the cycle diagnostic names its declarations.
An unknown, repeated, extra, or out-of-order binding rejects at the offending `fn_bind` and its complete extent.
A missing binding rejects at the complete `actual_decl`.
A signature, region-bound, effect-coverage, structural-contract mismatch rejects under FN-4 at the offending `fn_bind`, or at the raw function argument when no named group is involved.
Unresolved names retain their earlier resolver-owned rejections; no binding check guesses their meaning.
The post-resolution order among independent candidates remains the implementation-defined deterministic order above.
An invalid group publishes neither a partial interface nor a partial binding vector.

For a call to a callee class that carries written arguments — a user-generic `fn` [FN-2], or a retained-argument table operation [TYPE-5] — a missing, wrong-kind, wrong-count, or wrong-domain argument, or a missing operand, uses `SourceNode` at the `call` node and that node's complete source extent.
For an operation spelled infix, a wrong operand domain or a missing operand uses `SourceNode` at the `infix` node and its complete extent.
An extra operand or every wrong exact operand type other than the TYPE-7 implicit-read case uses `SourceNode` at the first offending `atom` node in source order and that atom's complete extent — for [OP-2]'s operand-agreement error, the second operand atom.
The cited rule is the rule selected by the callee's class: [FN-2] for a user-generic call, and, for a table operation, the rule [OP-2] selects — OP-1 or TYPE-5.
The TYPE-7 case follows that rule and the general participating-node location above.
A table-operation call written with a `fieldinit_list` instead of positional operands cites GRAM-11 using `SourceNode` at the `call` node and its complete extent.
A result mismatch is located and attributed only by the consuming construct as stated in OP-2.

An [FN-8] ordinary-call requirement judgment begins only after every earlier callee, concrete-instantiation, argument, type, borrow-feasibility, and actual-expression-obligation judgment named by FN-8 succeeds.
An unproved or refuted instantiated goal is one hard rejection citing FN-8 with `SourceNode` at that existing `call` node and `SourceCoordinate` equal to the call node's complete checked half-open source extent.
Its deterministic payload contains the concrete callee instance, the failing `requires_clause` NodePath, the complete instantiated typed goal, and exactly one disposition, `unproved` or `refuted`.
The required restructuring is `establish the complete callee requirement with one dominating branch or one preceding proved invariant before the call`.
When the payload contains an occurrence-local call-argument evaluated-value datum, it additionally renders that datum as `argument #N pre-transfer value`, with N the zero-based argument ordinal, and replaces the restructuring with `bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow`.
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
Instances use DIAG-1's stable concrete-instance order, selected returns use NodePath order, and the first relation failure wins.
No FN-9 failure fabricates an executable epilogue, runtime fallback, optimizer assumption, pending named-outcome fact, or caller-side rejection.
An excluded caller route, including a named or pending outcome, is not itself a rejection: it establishes no S12 fact or metadata, and any later query that needed that absent relation is diagnosed only at that later node by its ordinary owning rule.

Invariant and certificate diagnostics use their ordinary semantic schedule.
FN-1 first rejects every structurally unreachable statement; only a reachable loop header or `invariant_stmt` enters the schedule below.
After GRAM-4 and INV-1 have admitted the invariant names and their uniqueness, INV-1 checks for one ordinary or counted loop header its affine formation, the simultaneous base batch, every reachable arbitrary-backedge batch, and then any counted exact-exhaustion export, in that order.
For one local `invariant_stmt`, INV-1 first admits its name and then checks target formation.
For an optional block, ordinary parsing and lexical resolution precede semantic checking; PRF-1 then selects a rejection in this precedence: for each `proof_use` in source order, factor canonicality and then relation-source formation; whole-block redundancy; the 4096-entry capacity, duplicate normalized sources, and checked scaled-sum formation; every written `proof_use` independently against the one entering context in source order; then the one final DIRECT residual.
The first failed premise or target owns the rejection at the smallest source node fixed by INV-1 or PRF-1.
No written invariant conclusion enters the context before its complete owning judgment succeeds, and no later invariant may supply evidence to an earlier one.
Complete OP-2, OP-4, OP-9, FN-8, FN-9, layout, address, and target-domain judgments select their own ordinary source errors.
PAR-1 and PAR-2 permission failures select the sequential checked lowering or an explicit unsupported target lowering, never a source rejection.
An unavailable semantic judgment or inconsistent internal derivation is a compiler failure or explicit unsupported capability, not a guessed source rejection.

A mechanical fix or restructuring is included exactly where the owning rule requires one.
Every published static diagnostic is deterministic for one compiler executable under the conditions above.
Cross-implementation byte identity is required only where this specification explicitly fixes both selection and encoding.

[DIAG-2] Successful semantic checking produces one private checked-program value bound to the exact canonical compilation unit.
It is the only input that may grant lowering authority.

The checked program explicitly represents every source operation and every compiler-derived operation required for execution, including drops, arena releases, monomorphized instances, propagation edges, every direct slice value's finite ownership-origin set, every `own slice` result's FN-1 formal return-origin ceiling and call-site substitution, and one abstract target-domain representability obligation at every runtime-sized allocation and element-address operation governed by [STOR-6].
It retains every [FN-8] GoalTemplate, its requirement occurrence `(concrete callee instance, requires_clause NodePath)`, every concrete call substitution and discharged-goal derivation, every proved body-entry requirement, and each inhabited or contradiction-proved body disposition.
It retains every proof-required integer-domain, allocation-fit, subscript-bounds, layout, address, and target-domain obligation occurrence together with the exact derivation authorizing its accepted source node.
It separately retains each successful PAR-1 and PAR-2 permission derivation that authorizes an optional nonsequential lowering; absence retains no permission and changes no source verdict.
It also retains every proved loop-invariant base and arbitrary-backedge judgment, each permitted exhaustion export, and every PRF-1 premise-admission, factor, scaled-sum, and final-DIRECT-residual judgment.
Target lowering must discharge each target-domain obligation from the selected target plus already-checked layout, allocation, and bounds facts before emitting the governed allocation or address operation; it may not replace a missing proof with a runtime guard.
No accepted proof-required operation carries an implicit runtime check or elimination disposition: a subscript, exact integer operation, buffer allocation, or function range requirement is `discharged` at its owning source node, and the checked program retains its exact [ENT-4] or [ENT-6] derivation there.
A concrete terminal-root identity uses the owning function instance plus the operation NodePath/family/conjunct, the call NodePath/callee/requirement NodePath, or the complete-postcondition block/relation ordinal; display symbols are never identity.
A `requires_clause` is represented only by its GoalTemplate, call-site derivations, and proved body-entry fact; an `ensures_clause` only by its verified RelationTemplate, selected-exit judgments, and derivations.
Neither contract clause has executable checked-program form.
Facts-off compilation preserves every source-acceptance and call-goal judgment and erases the same proof-only syntax before lowering.
Neither a discharged call goal nor a proved body-entry fact authorizes `llvm.assume`, an optimizer fact, or a second lowering path.
STOR-6 target-domain obligations instead follow the target-stage discharge judgment above identically in facts-on and facts-off compilation; an optional optimizer fact supplies no target-layout discharge.

The one current ProofContext is failure-atomic.
No fact, postcondition summary, invariant target, partial-operation discharge, checked function, or lowering input leaves semantic scratch until every premise of its originating judgment has succeeded.
Every `requires` fact enters a callee only after the caller has discharged that concrete call's complete instantiated requirements; runtime argument values establish nothing by themselves.
Every `ensures` summary of one call-graph strongly connected component is withheld until every selected return of every concrete member has been proved from its own entry facts and body flow, after which the component publishes its complete summaries atomically.
No member of that component may use a summary withheld by this rule, so recursive postconditions cannot bootstrap one another without an independently established source fact.
A header invariant enters the current loop-body ProofContext as an induction hypothesis only after its complete simultaneous base batch succeeds; it grants no checked-program or continuation authority unless every applicable arbitrary-backedge batch also succeeds. A local invariant enters its dominance region only after its one AUTO or complete PRF-1 judgment succeeds. A PRF-1 conclusion enters only after every source, factor, scaled sum, and final DIRECT residual succeeds.
Any failure discards the complete prospective checked program and every unpublished derivation root.

The current ProofContext and its diagnostic derivations are produced by the same source-semantic walk.
Every accepted fact has one specification-enumerated constructor and the direct parents used by that constructor; no runtime-value origin, compiler-generated record, optimizer result, or written conclusion supplies an alternate route into the context.
Every parent precedes its child, every retained node is reachable from a required accepted root, and finalization performs one reachability traversal and one identity remap.
An implementation may choose its private Rust layout, but it may not create another acceptance-bearing fact view, omit a source fact and reconstruct it after publication, re-run the function under a mask, or consult a second checker.
A callee summary is referenced by checked-program-private `(concrete callee instance, postcondition occurrence)` identity; a caller never imports a callee's local node identity.

Every new S7 fact is retained even when no later query consumes it.
`BitAndBound` roots the exact direct `iand` result relation at its binding and carries the selected unsigned operation row, result binding, operand ordinal, admitted operand term or constant, and source event.
`ShiftOneNonzero` roots the exact direct `ishl.wrap` result disequality against the mathematical-zero endpoint Z and carries the selected unsigned row, result binding, count atom, and the checked mathematical-one constant identity.
`UnsignedDivisionBound` roots the direct exact-division relation `q <= a` and carries the selected unsigned row, result binding, admitted dividend term or constant, positive written divisor value, and source event; [ENT-6]'s `k*q <= a` automatic affine image cites this same root together with the exact q and a value images rather than creating an independent source fact.
`UnsignedRemainderBound` roots the direct exact-remainder relation `r < d` and carries the selected unsigned row, result binding, admitted divisor term or constant, and source event.
Each `SignedRemainderBound` roots one endpoint of the direct signed-remainder interval and carries the selected signed row, result binding, checked constant divisor, minimum-or-maximum endpoint identity, and source event.
A signed row where unsigned is required, non-direct result, nonterm required operand, zero or unavailable constant, or non-one shift source forms no corresponding root.
These are ordinary source roots in the same DAG, not trusted optimizer facts.

For every concrete FN-9 declaration, `PostconditionExit` roots each discharged selected-return relation on the exact local [ENT-4] derivation after result substitution and before return transfer or cleanup.
It also retains the entry-image-stability disposition and ordered invalidating event when unavailable; successful absence of an invalidating event is diagnostic metadata, not a fabricated positive parent.
`PostconditionAggregate` has the nonempty selected-exit roots as parents in return-NodePath order.
A non-discharged declaration retains its ordered dispositions and residual but no success aggregate root.
Component summaries become referenceable together only after the SCC schedule validates that every summary reference points strictly from a caller component to an earlier callee component.

Every S12 fact actually established in accepted semantic flow is a required caller-local root even when no later query consumes it.
`PostconditionCall` carries q, the checked aggregate summary reference, exact per-formal pre-transfer substitution, A0's complete actual-obligation and FN-8 goal roots, and the ordered transfer/consume/borrow/effect/kill event prefix.
`PostconditionDirectResult` adds the fresh ordinary-let binding substitution.
`PostconditionDirectMatch` adds the direct-call scrutinee, selected `Ok` variant and `value` field identities, and payload substitution at arm entry.
`PostconditionDirectReceiver` adds the direct-set target kill and result-only post-write substitution.
`PostconditionSelectedReceiver` adds the selected arm's immediate payload read, target kill, and result-payload-only outer substitution.
A named or pending outcome, false `M(c,q)`, rejected call, killed support, or excluded receiver creates no fact root or pending metadata.

For bounded `value_if` delivery, `PostconditionGive` records one eligible reaching edge, the already evaluated source value and relation root, then the forward `d ↦ x` substitution, then that edge's ordinary scope and event kills applied to every other support in that order.
`PostconditionDeliveryJoin` orders all non-contradictory reaching delivery images by edge NodePath and applies exactly the ordinary [ENT-5] L0 delivery join.
Its parents therefore need not state byte-identical relations; an `x < 8` image and an `x < 128` image may parent the joined `x < 128` root.
Contradictory inputs use the existing contradiction root and are neutral when a non-contradictory input reaches.
Missing edge evidence, a `value_match`, or no common joined relation creates no delivery root.
Kill events never become invented positive evidence.

Candidate S12 and delivery nodes live only in failure-atomic semantic scratch until the current source judgment and its ordinary ownership, effect, and kill events all succeed.
Any failure discards the whole candidate root set with the unpublished checked program; success commits that set once, without a provenance batch, masked rerun, strict gate, or reconstructed root.
This preserves A0 and S12 atomicity and makes the derivation DAG an explanation of the accepted semantic flow rather than a second acceptance path.

For every `for_stmt`, the checked program additionally represents its optional source label and mandatory binder, the two source endpoint atoms in evaluation order, the two immutable compiler-owned captures with their identities, binder initialization, the pure header comparison, both header edges, the exact normal-body cleanup and update order, every no-update exit edge, the distinct header and continuation carried-binding sets, every hidden scope kill, and the complete [ENT-4] derivation of each S11 fact.
These are checked semantic operations and facts, not a source desugaring or an optimizer reconstruction.

Function-kind instantiation retains the source identity of each written parameter, group application and member, its complete concrete substitution, the selected ordinary function instance, and its FN-4 signature, row and contract checks. Every bound call has one direct target and its instantiated formal interface; lowering emits no group object or adapter.
A symbolic template may use a `FunctionFormalContract` hypothesis while checking its written spelling. That source-local hypothesis records the formal contract and relation ordinal, is never a proof of an actual body, and is never published in a concrete checked program or its derivation DAG. Concrete calls use only their selected actual's independently verified FN-9 summaries. No algebraic-law metadata or law-derived optimization authority exists.

The checked-program representation is private compiler state.
Its Rust layout, allocation strategy, dense identities, instruction grouping, and internal ordering where this specification defines no semantic order are implementation-defined.
Any diagnostic or debug projection is explanatory only: it establishes no source fact and grants no lowering authority.

The language defines no writer-reachable runtime proof-failure report, because every writer-visible proof obligation is discharged before lowering or rejects the source.
Static source diagnostics follow [DIAG-1]. Diagnostic derivations follow [DIAG-2] and explain the source semantic decision; they are never reloaded or checked as a second acceptance step.
An implementation may report unavailable resources, trusted-computing-base failures, or compiler failures on implementation-defined channels, but none is a source-language outcome and none may be mistaken for a successful source judgment.

## 13. Execution overlap

[CAP-1] The kernel defines no writer-visible capability category and no additional concurrency permission. `own`, `&`, `&uniq`, place overlap, and the ordinary effect row are the complete authority and interference vocabulary available to [PAR-1] and [PAR-2].
This version defines no thread construct. A later thread construct must derive transfer and sharing permission from these same ownership rules and the represented type; it may not add hidden shared mutation to an opaque nominal. Data-race impossibility is D1 law; general race conditions are out of scope (C004 amended scope).

[PAR-1] An implementation may execute two statements of one block with overlapping execution only when the permission this rule defines holds for that ordered pair.
Permission holds for the ordered pair (s1, s2), where s1 precedes s2 in one block, exactly when all of the following hold.
Each of s1 and s2 is one call of a declared function [FN-1], written either as a `let_stmt` whose selected `ordinary_let_rhs` is that call or as the scrutinee of a `match_stmt` or of a `let_stmt` selecting `value_match` or `value_if`; a scrutinee member is judged exactly as a `let`-bound member is and is the last member of any chain it belongs to, because its arm blocks are not statements of the enclosing block; a recursive or mutually recursive user callee is admitted on the same terms as any other.
No argument of s2 reads a binding s1 defines.
The two calls have disjoint footprints under [OWN-7]: one call's written footprint is the places its callee row's `writes` paths reach through its actual arguments under the [EFF-2] call-boundary projection, together with the places its consumed `own` arguments name and the caller region each `allocates(arena 'r)` entry names after region substitution, and its read footprint is the places that row's `reads` paths reach under the same projection; the written footprint of s1 overlaps neither footprint of s2, and the written footprint of s2 overlaps neither footprint of s1.
Evaluating a statement's own argument expressions is part of that statement and therefore part of the overlap, so each call's written footprint also overlaps no place the other statement's argument expressions read; taking the address of a place is not reading it, and both directions are required because which statement's argument evaluation an overlap moves is the implementation's choice.
Each statement additionally holds, for the duration of its call, a loan on the resolved place of every argument written as a borrow [OWN-12]: a `&uniq 'r` argument holds an exclusive loan and a `&'r` argument a shared loan, whatever that argument's parameter region does or does not carry in the callee's row.
A loan of one statement denies permission against an overlapping loan or footprint element of the other exactly where [OWN-5] denies the corresponding pair of a live loan and an overlapping access: an exclusive loan denies against every overlapping loan, written or read footprint element, and argument-expression read of the other statement, and a shared loan denies against every overlapping exclusive loan and written footprint element; two overlapping shared loans deny nothing.
The reason is [OWN-5] itself: every borrow this rule judges is live and usable across the whole of its statement's call [OWN-12], so an implementation that overlaps two statements makes both statements' borrows simultaneously live and usable, and permission therefore requires of the resulting loan state exactly what [OWN-5] requires of one statement holding all of those loans at once.
A footprint element whose caller place the implementation does not resolve overlaps every place, and so does a place read by an argument expression whose caller place the implementation does not resolve, and so does the loan of a borrow-written argument whose caller place the implementation does not resolve, so an unresolved element denies permission rather than granting it.
Every statement written between s1 and s2 is part of the permitted window and is judged by these same conditions: it is an ordinary value `let_stmt`, a `let_stmt` whose selected right-hand side is one call judged exactly as a member is, or a `set` or `replace` statement; its written, read, and argument-expression footprints and its loans are formed exactly as a member's are; and no written footprint, read footprint, or loan of a member may stand against any other window statement, nor any intervening statement's against a member, where the conditions above deny the pair; two intervening statements owe each other nothing, because both admitted schedules run the intervening statements in source order on the thread that did not take the hand-out, so no two of them ever overlap — with one exception on the member side, owed to the same pair of admitted schedules as the argument-evaluation sentence above: no obligation runs between an intervening statement's written footprint or loan and s1's own argument-expression reads, because either admitted schedule completes s1's argument evaluation before any intervening statement runs.
A statement of any other form between the members denies permission, a statement carrying an exit edge denies permission, and a non-call statement that forms a borrow denies permission, so every loan live inside a permitted window is one an argument of a judged call holds.
Every argument loan remains live through the ordinary call's return. An implementation may overlap a permitted window only while retaining the same call boundary and all of its live storage; otherwise it executes the window sequentially.
Every normal continuation of s1 reaches s2, so no edge out of s1 leaves the enclosing block or function without first reaching s2, and every normal continuation of each intervening statement likewise reaches s2.
Permission for a chain of three or more statements is exactly permission for every ordered pair the chain contains.

Under a permitted overlap, bindings and every Whitefoot state place equal the source-order result.
That identity is conditional on contract compliance, exactly as [SCOPE-3]'s freedom from undefined behavior is conditional on its trusted computing base.
It holds in every source execution, not in a typical execution or in some execution: accepted source contains no writer-reachable proof-failure branch, and every partial operation in the window has already been discharged by its owning static Goal.
No overlapped pair reaches one state place or violates one ordinary loan except as the permission conditions above admit.
Target-resource exhaustion and trusted-computing-base termination remain outside the source execution model under [SCOPE-3] and grant no overlap permission.
No permission or execution path reads a proof-failure latch or pays any other cost for a writer-reachable runtime proof fallback.
The number of workers, the identity of the host thread that executes a statement, the schedule, and whether an overlap was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Every construct of this specification defines one total sequential order over its operand evaluations, and this rule is a consumer of that order rather than a relaxation of it.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification.
The counted permission [PAR-2] forms every statement footprint and loan exactly as this rule does.

[PAR-2] An implementation may execute two iterations of one `for_stmt` body with overlapping execution, and may recombine that loop's accumulator across them, only when the permission this rule defines holds for that counted loop.
Permission holds for a `for_stmt` L exactly when all of the following hold, writing B for L's body and forming every written, read, and operand-read footprint of a statement of B exactly as [PAR-1] forms one.
Among whole-place writes of B, at most one place is rooted in a binding declared outside L; that binding is L's accumulator, and every occurrence of it in B is one operand of one `set` statement whose target is that whole binding and whose right-hand side is one operation applied to that operand and to a second operand reaching the accumulator nowhere.
That operation is one operation fixed for the accumulator across the whole of B, and is exactly one of `+wrap`, `*wrap`, `iand`, `ior`, `ixor`, `imin`, `imax`, `band`, `bor`, and `bxor` [OP-1].
Every place a footprint of B writes is either that accumulator's whole place, is rooted in a binding B itself introduces, or is one proved single-binder affine element write defined below.
A proved single-binder affine element write is exactly a `set_stmt` whose target is one direct array or buffer subscript rooted in an own binding declared outside L or reached through the live usable `&uniq` holder that made that target writable [OWN-5], whose exact [OP-4] bounds obligation at that subscript is discharged in the current ProofContext and retains the offset's canonical exact value `a*i + b`: i is L's compiler-owned binder, a and b are mathematical integer constants, a is nonzero, and no other symbolic term occurs.
The retained [OP-4] result and affine value are consumed from the same source semantic check. The value may have been carried through copies and checked affine operations; PAR-2 neither repeats the bounds proof, reconstructs the value from parser shape, nor trusts a runtime check, optimizer fact, or backend result.
For permission only, this fixed form refines the ordinary whole-collection write footprint to the single-element range `[a*i + b, a*i + b + 1)`.
The counted recurrence of [FN-1] gives distinct binder values to distinct iterations, and multiplication by the same nonzero integer a preserves distinctness, so their refined ranges do not overlap; statement order within one iteration is unchanged.
This refinement proves only the source element-range and cross-iteration disjointness. The selected-target [STOR-6] check must still prove the concrete element stride, layout, and address domain before emission; that later target check consumes the already-permitted source access and never grants PAR-2 permission retroactively.
Every write by B to one mapped root must be another proved single-binder affine element write carrying exactly the same a and b; different resolved roots may carry different maps. Every operand read through that same root binding must be a direct array or buffer subscript whose own discharged [OP-4] result retains exactly the same a and b. For permission only, that read footprint is refined to the same single-element range, so it overlaps writes of its own iteration in source order and no access of another iteration. A whole-root read, a subscript carrying a different or unavailable map, any shared or exclusive loan overlapping the resolved root, or an unresolved place denies.
Thus this version admits one affine map per root, including same-index read-modify-write and writes reached through a live usable `&uniq` holder. A constant image, a `replace_stmt`, a stencil, a whole-root read or write, a callee-projected access, two different affine maps of one root, and every other range or injectivity argument deny permission rather than starting proof search.
Every place a footprint of B holds an exclusive loan on — its statements' argument borrows holding loans exactly as [PAR-1]'s do — is rooted in a binding B itself introduces, so no two iterations hold exclusive loans on one place.
Apart from the mapped-root prohibition above, a shared loan needs no condition of its own, because the accumulator is the only other enclosing place any iteration writes and an accumulator any borrow reaches is refused by the accumulator condition; a non-call statement of B that forms a borrow denies permission, exactly as one denies a [PAR-1] window.
A footprint element whose caller place the implementation does not resolve overlaps every place, so an unresolved element denies permission rather than granting it.
Every argument loan remains live through its ordinary call's return. Effects and ordinary loans decide interference between iterations exactly as they do between PAR-1 calls. An implementation retains each iteration's live storage for that complete extent.
Every normal continuation of every statement of B reaches L's compiler-owned binder update, so no statement of B is a `return_stmt`, a `give_stmt`, a `break_stmt` resolved to L or a loop enclosing L, or a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3].

Under a permitted overlap every state-place observable is the one produced by executing L's iterations in index order.
Write a0 for the accumulator's value on the true header edge entering the first executed iteration, and t0 through tm for the values the second operand of its writes evaluates to, in the order those writes execute across L's iterations taken in index order.
Source order computes the accumulator's value at L's continuation as the left-nested application of that operation to a0 then t0 through tm where its writes place the accumulator in the first operand position, and as the right-nested application to t0 through tm then a0 where they place it in the second.
An implementation may instead apply that operation over any binary tree whose leaves are a0 and t0 through tm, each occurring exactly once and in any order, together with any number of leaves holding that operation's identity element.
Every admitted operation is a total function on the complete value set of its type, carries no domain obligation, and is associative and commutative on that set with a two-sided identity element — `+wrap` and `*wrap` are the ring operations of the integers modulo two to the width, with identities zero and one; `iand`, `ior`, and `ixor` are the meet, join, and group operations of the bit vector, with identities the all-ones vector, zero, and zero; `imin` and `imax` are the meet and join of that type's total order, with identities the type's greatest and least values; and `band`, `bor`, and `bxor` are the two-element cases of the same three, with identities `true`, `false`, and `false` — so every such tree denotes one value of that type and the accumulator's value at L's continuation is that one value in every execution.
No further operation is admitted: `+`, `+defined`, and `+checked` each attach a domain obligation or a `Result` route to every application, `+sat` is not associative, and no float operation of [OP-1] is associative, so recombining a `fadd.strict` or `fmul.strict` fold could change published bytes.
This rule uses associativity, commutativity, and the identity together: commutativity is what admits any leaf order and the fold of the second operand position, and the identity is what lets an implementation seed a subrange of iterations before knowing whether that subrange writes, so a range of iterations that writes the accumulator not at all contributes either nothing or identity leaves that change nothing.
That identity is conditional on contract compliance exactly as [PAR-1]'s is; every partial operation in the admitted loop has already been discharged before lowering.
Both endpoint atoms are still evaluated exactly once each in [FN-1]'s order before any iteration begins, and the binder still takes each value of the half-open range exactly once; this rule relaxes only the order in which iterations execute and the shape of the accumulator's combination, never the set of iterations, the values the binder takes, or either endpoint evaluation.
The number of workers, the identity of the host thread that executes an iteration, the schedule, how the index range is divided, and whether any overlap or recombination was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
When an execution of one iteration does not reach its continuation, the overlapped execution produces exactly the observables the index-order execution produces before that point and produces none after it.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Permission over the iterations of a `for_stmt` written inside B is exactly this rule applied to that loop; no rule of this specification joins two index ranges into one iteration space.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification for the accumulator or any other place.

## 14. Prelude (normative, counted)

[PRE-1] The prelude contributes ordinary nominal, constructor, numeric-bound and function declarations to every compilation unit. Their source visibility, whole-unit collisions, typing, ownership and calls are the ordinary rules; an entry's prelude origin supplies only its deterministic diagnostic ordinal [TYPE-6, DIAG-1].

The following opaque nominal declaration records carry no public constructor, fields, variants, region parameters or type parameters. This table is declaration notation, not additional source syntax. An opaque nominal is a bare TYPEID under GRAM-3, region-free under STOR-5, and neither copy nor const-eligible. OP-9 gives every such nominal the same conservative layout ceiling. Its drop is empty; explicit linearity and ordinary ownership closure are exactly PROV-6.

| Nominal | Modifier |
|---|---|
| Args | none |
| HostString | none |
| RelativePath | none |
| DirectoryRead | linear |
| ReadFile | linear |
| OutputStream | none |
| ExitStatus | none |
| DirectorySource | linear |
| HandleFactory | none |
| InputStream | none |
| SocketAddress | none |
| TcpListener | linear |
| TcpReceive | linear |
| TcpSend | linear |

The complete ordinary struct and enum declarations are:

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

struct TcpConnection {
  receive: TcpReceive;
  send: TcpSend;
}

struct Inputs {
  args: Args;
  cwd: DirectoryRead;
  stdout: OutputStream;
  stderr: OutputStream;
  handles: HandleFactory;
  stdin: InputStream;
}

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

enum ReadStop {
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
enum ListStop {
  ListEnd();
  ListFailed(error: IoError);
}
enum FileOpenOutcome {
  FileOpened(value: ReadFile);
  FileOpenFailed(error: IoError);
}
enum DirectoryOpenOutcome {
  DirectoryOpened(value: DirectoryRead);
  DirectoryOpenFailed(error: IoError);
}
enum SourceOpenOutcome {
  SourceOpened(value: DirectorySource);
  SourceOpenFailed(error: IoError);
}
enum ListenOutcome {
  Listening(listener: TcpListener);
  ListenFailed(error: IoError);
}
enum AcceptOutcome {
  Accepted(connection: TcpConnection, peer: SocketAddress);
  AcceptFailed(error: IoError);
}
enum ConnectOutcome {
  Connected(connection: TcpConnection);
  ConnectFailed(error: IoError);
}
```

`TcpConnection` and `Inputs` have ordinary public constructors, fields, partial-move and destructuring rules. Their linearity follows their fields. No relation between two fields is implied by constructing either struct.
The two built-in numeric bounds `Int` and `Float` admit exactly OP-1's integer and floating-point domains and imply `copy` under PROV-6. They are not source declarations, formal groups, implicit behaviors or logical-law bundles; a source actual cannot bind or extend either bound.

The complete function declarations are the following ordinary GRAM-2 `fn_sig` records. A record's final semicolon is table punctuation, not a new top-level source production. Each signature uses ordinary parameter regions under FORM-8, ordinary parameter paths under EFF-1, and the same requirement and postcondition templates as any FN-8/FN-9 contract. No proposition is available merely from a function's name, implementation, result constructor, or prelude origin.

```
fn args_count(args: &Args) -> result: own u64 reads(args);
fn arg_get(args: &Args, position: own u64) -> result: own Result<HostString, ArgError> reads(args);
fn host_bytes_len(value: &HostString) -> result: own u64 reads(value);
fn host_copy_bytes(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn host_utf8_len(value: &HostString) -> result: own Result<u64, Utf8Error> reads(value);
fn host_copy_utf8(value: &HostString, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads(value, destination), writes(destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn relative_path(value: own HostString) -> result: own Result<RelativePath, PathError> pure;
fn open_read(factory: &uniq HandleFactory, root: &DirectoryRead, path: &RelativePath) -> result: own FileOpenOutcome reads(factory, root, path), writes(factory);
fn read_at(factory: &uniq HandleFactory, file: &uniq ReadFile, destination: &uniq MutSlice<u8>, file_offset: own u64, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(factory, file, destination), writes(factory, file, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn write_once(factory: &uniq HandleFactory, output: &uniq OutputStream, source: &Slice<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(factory, output, source), writes(factory, output) contract {
  requires start <= end;
  requires end <= len_of(deref(source));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn exit_status(code: own u8) -> result: own ExitStatus pure;
fn open_directory(factory: &uniq HandleFactory, root: &DirectoryRead, name: &Slice<u8>, start: own u64, end: own u64) -> result: own DirectoryOpenOutcome reads(factory, root, name), writes(factory) contract {
  requires start <= end;
  requires end <= len_of(deref(name));
};
fn open_directory_source(factory: &uniq HandleFactory, directory: &DirectoryRead) -> result: own SourceOpenOutcome reads(factory, directory), writes(factory);
fn directory_next(source: &uniq DirectorySource, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> (result: own Result<unit, ListStop>, next: own u64, entries: own u64) reads(source, destination), writes(source, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures start <= next;
  ensures next <= end;
};
fn open_file(factory: &uniq HandleFactory, root: &DirectoryRead, name: &Slice<u8>, start: own u64, end: own u64) -> result: own FileOpenOutcome reads(factory, root, name), writes(factory) contract {
  requires start <= end;
  requires end <= len_of(deref(name));
};
fn close_read(factory: &uniq HandleFactory, file: own ReadFile) -> result: own Result<unit, IoError> reads(factory, file), writes(factory, file);
fn close_directory(factory: &uniq HandleFactory, directory: own DirectoryRead) -> result: own Result<unit, IoError> reads(factory, directory), writes(factory, directory);
fn close_directory_source(factory: &uniq HandleFactory, source: own DirectorySource) -> result: own Result<unit, IoError> reads(factory, source), writes(factory, source);
fn read_next(factory: &uniq HandleFactory, input: &uniq InputStream, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(factory, input, destination), writes(factory, input, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn socket_address_v4(a: own u8, b: own u8, c: own u8, d: own u8, port: own u16) -> result: own SocketAddress pure;
fn socket_address_v6(a: own u16, b: own u16, c: own u16, d: own u16, e: own u16, f: own u16, g: own u16, h: own u16, port: own u16) -> result: own SocketAddress pure;
fn tcp_listen(factory: &uniq HandleFactory, address: &SocketAddress) -> result: own ListenOutcome reads(factory, address), writes(factory);
fn tcp_accept(factory: &uniq HandleFactory, listener: &uniq TcpListener) -> result: own AcceptOutcome reads(factory, listener), writes(factory, listener);
fn tcp_connect(factory: &uniq HandleFactory, address: &SocketAddress) -> result: own ConnectOutcome reads(factory, address), writes(factory);
fn receive_next(receive: &uniq TcpReceive, destination: &uniq MutSlice<u8>, start: own u64, end: own u64) -> result: own Result<u64, ReadStop> reads(receive, destination), writes(receive, destination) contract {
  requires start <= end;
  requires end <= len_of(deref(destination));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn send_once(send: &uniq TcpSend, source: &Slice<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(send, source), writes(send) contract {
  requires start <= end;
  requires end <= len_of(deref(source));
  ensures when Ok(value: next): start <= next;
  ensures when Ok(value: next): next <= end;
};
fn close_listener(factory: &uniq HandleFactory, listener: own TcpListener) -> result: own Result<unit, IoError> reads(factory, listener), writes(factory, listener);
fn close_receive(factory: &uniq HandleFactory, receive: own TcpReceive) -> result: own Result<unit, IoError> reads(factory, receive), writes(factory, receive);
fn close_send(factory: &uniq HandleFactory, send: own TcpSend) -> result: own Result<unit, IoError> reads(factory, send), writes(factory, send);
```

Each record is an ordinary callable boundary usable by a direct call or a function-kind binding under FN-2 through FN-5. Its definition is supplied by the build and must satisfy the declared boundary [SCOPE-3]; calls neither inspect nor classify that definition. There is one ordinary callable ABI for definitions written in Whitefoot and definitions supplied by linking. A loan passed to either lasts through that call's return under the same OWN rules. A missing definition or incompatible physical representation is a build/link failure, not a source-language rejection.
PRE-1 requirement templates are discharged by FN-8 and verified declaration postconditions are instantiated only by CALL-6 and FN-9's ordinary selected-result rules. The supplied definition is responsible for those propositions; no compiler-owned operation fact or alternative acceptance judgment exists.
The declaration preorder is opaque nominals in table order, then each struct or enum above in written order with its constructor or variants and their fields in declaration order, then `Int`, `Float`, and each function above with its region and value parameters in declared order. Owner-local fields and parameters do not enter compilation-root name lookup. This preorder fixes each PRE-1 diagnostic ordinal [DIAG-1].

## 15. Obligation discharge: deterministic facts, invariants, and local certificates (normative)

[ENT-1] The entailment fragment is a closed, deterministic, terminating derivation system fixed completely by this specification.
Its state is the L0 relation state, [ENT-2]'s finite signed opaque goals, [ENT-6]'s exact current-value images and specification-fixed automatic affine images, and the finite affine theorems admitted by [INV-1] and [PRF-1].
Complete-state obligation discharge [ENT-6], ordinary-call requirement discharge [FN-8], verified normal-return proof [FN-9], loop induction and program-point invariant checking [INV-1], and local certificate checking [PRF-1] are post-resolution source-acceptance judgments under [DIAG-1].
They are identical in facts-on and facts-off compilation and are not an optimizer-fact family.

The fact sources are exactly the executed control-flow edges, independently proved function requirements at callee entry, declaration and type properties fixed by this specification, constants, compiler-owned structural consequences enumerated by [ENT-3], verified earlier-SCC normal-result publications [FN-9], and machine-proved header or local invariant targets.
A runtime-origin value is an ordinary typed term in those judgments; its origin is neither a fact source nor a reason to discard an otherwise derived fact [SCOPE-2].
Only the fact sources enumerated above establish propositions; a written conclusion, unselected condition, diagnostic record, or optimizer result does not.

No source postcondition is trusted: FN-9 proves every selected exit, requires a nonempty selected-exit set, and withholds same-SCC summaries before atomic publication.
The fragment is the deterministic checker derivation of [OP-2], [OP-4], [OP-9], [FN-8], [FN-9], [INV-1], [PRF-1], [STOR-6], and [DIAG-2] for the judgments this version attaches.
A solver result never participates, and no implementation may strengthen, weaken, time-bound, randomize, or truncate an unsuccessful query within the derivable set.
Every semantic candidate family and iteration count is fixed below from the complete source text; an unproved result requires exhausting its complete family regardless of elapsed time, machine speed, thread schedule, hash iteration order, or memory pressure short of [SCOPE-3]'s external resource boundary.
A successful query may retain the first witness in the specification-fixed order and omit later witnesses, because no later candidate can revoke that success; this changes diagnostic parent choice only, never the derivable set or acceptance.
An implementation may organize or cache the same derivation differently, but exceeding a wall-clock or cumulative-work budget is never a source-language verdict.
Parser, finalizer, or canonical-source invocation ceilings may stop one compiler invocation only as [DIAG-1]'s non-language `Resource` failure; they return neither source acceptance nor source rejection and may never turn an unfinished ENT-1 candidate family into `unproved`.
Two conforming implementations derive the same fact state at every applicable program point; the same FN-9 selected exits, concrete-SCC order, and established result relations; the same certificate premise, combination, and target dispositions; and the same disposition for every operation obligation, call goal, postcondition relation, and invariant.

Every nongeneric source body receives this judgment whether or not the selected invocation reaches it.
Every generic source body additionally receives one source-schema judgment under the one source-canonical symbolic substitution formed during generic-body validation, even when it has no concrete instantiation.
That schema checks every OP-2/OP-4/OP-9, FN-8, and expressible FN-9 judgment its symbolic vocabulary can represent; an unproved operation is not accepted merely because no concrete instance is reachable.
Generic integer and float type parameters are copy datums only for exact opaque goals in this schema and are not [ENT-2] L0 fragment types, while an integer-typed const generic remains the symbolic constant term [ENT-2] fixes.
An FN-9 schema goal exists only when its result datum, selected return, and normalized relation are expressible in the finite schema vocabulary; otherwise it is rechecked in every inhabited concrete instance and is never approximated.
The schema publishes no executable function, callable summary, or lowering authority.
Every inhabited concrete [FN-2] instance is rechecked independently after substitution; a contradictory path discharges only by [ENT-4] and never bypasses an ordinary formation check.
If concrete instances disagree, the first invalid concrete instance in stable instance order rejects the shared source occurrence.

The fragment joins the trusted computing base exactly as the type and ownership checkers do [SCOPE-3]; a wrong derivation is a compiler defect owned by implementation repair and tests, not a second runtime validation layer.
Adding a fact source, relation family, closure rule, proof rule, protected operation family, or callable publication surface is an explicit specification amendment [META-5], never implementation strengthening.
[ENT-2] The fragment constructs one ProofContext for one concrete function body at a time.
No caller fact is copied into a callee: an ordinary call judges its instantiated [FN-8] goal in the caller's entering state, the callee body begins with its own proved requirement as [ENT-3] source S4, and only a separately FN-9-verified earlier-SCC summary may establish its instantiated normal-result relation back in the caller.
A fragment type is one member of the closed integer set [OP-2]; relations are over mathematical values, so relations between terms of different fragment types are well-formed and are created only by the sources and flow transports [ENT-3, ENT-5] admit.

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root `pbase` IDENT resolves to any `let_stmt` binding, a `for_stmt` binder, a `param`, any match binder regardless of its [OWN-13]-derived mode, or a named const [CONST-2], formed with any number of field-selection `psuffix`es and `deref` wrappings and no subscript suffix, whose final selected type is one fragment type; (b) a measure term `len_of(P)`, `cap_of(P)`, `room_of(P)`, or `head_of(P)` [MSR-1], of fragment type u64, where P is an admitted measure place — a `place` [GRAM-5] whose root resolves as in (a) and which is formed with any number of field-selection `psuffix`es, `deref` wrappings, and subscripts, and whose final selected type is a measured type [MSR-1] having that measure; (c) a constant — the mathematical value of an integer literal or of an integer-typed named const, or symbolically an in-scope integer-typed const-generic parameter; (d) one of the two compiler-owned u64 capture terms belonging to an admitted `for_stmt`, identified exactly by `(that for_stmt's NodePath, lower)` or `(that for_stmt's NodePath, upper)`; (e) the one compiler-owned symbolic result datum of an admitted FN-9 clause while its RelationTemplate is formed, identified by that `ensures_clause`, its route or unrouted class, and fragment type; (f) the one compiler-owned commit value of an admitted [SET-1] `set` whose right-hand side has one fragment type, identified exactly by `(that statement's NodePath, that fragment type)`; (g) the distinguished zero term Z, used only to carry constant bounds, S7's exact mathematical-zero disequality, and [ENT-6]'s normalized integer-domain components; or (h) one compiler-owned measure datum [MSR-3], which is a call datum [ENT-3.S13], identified exactly by `(that call's NodePath, the formal ordinal, that operand's ordered projections, whether it denotes the operand's value or one measure of it)`; an entry datum, identified exactly by `(the formal ordinal, that operand's ordered projections, which measure it denotes)`; or a placement datum, identified exactly by `(that statement's NodePath, which placement of [MSR-3]'s placement table it stands at, the ordinal within that statement, which measure it denotes)`.
The FN-9 result datum occurs only in its template: every selected-return or caller query substitutes it with one ordinary term or constant before flow, so it never enters a body state, survives a return, or creates runtime storage.
Two places are the same term exactly when their roots resolve to the same declaration event [TYPE-6, DIAG-1] and their canonical source spellings [FORM-2] are byte-identical; a fresh binding legally reusing an expired spelling is a distinct term, and distinct spellings are distinct terms even when they resolve to overlapping storage.
Term identity thus under-approximates aliasing, while kills [ENT-5] use [OWN-7]'s resolved-place overlap relation and over-approximate it.

After TYPE-5 succeeds, each `for_stmt` endpoint atom is admitted only when its evaluated value is itself one preceding term or constant.
An in-scope const generic is such a constant and is therefore an admitted endpoint [MSR-6].
Any other atom is a hard error citing ENT-2 at that endpoint's `atom` node, with `SourceCoordinate` equal to its complete checked half-open source extent and the restructuring `bind the computed u64 value with one preceding ordinary let and use that term as the endpoint`.
In particular a subscripted place is not made a term of clause (a) by endpoint position; clause (b)'s subscript admission is a measure place's admission and reaches no other position.
The two capture terms are finite, immutable, compiler-owned, and not source bindings or source places: source cannot name, write, borrow, move, or shadow them.
Their scope begins after their respective once-only endpoint captures and ends on every edge leaving the counted construct.
The counted binder's compiler fact scope begins at its initialization and ends on every edge leaving the counted construct, even though [TYPE-6] makes its source name visible only in the body.
A commit value is compiler-owned and unwritable in the same sense, and denotes the one value its `set` occurrence's right-hand side evaluated to: it exists from that evaluation, no [ENT-5] event kills it, and no later write can retarget it.
A call datum is compiler-owned and unwritable in the same sense, and denotes the value one `own` operand of a declared relation had at its call's pre-transfer point [ENT-5]: it exists from that point, contains no place, and no [ENT-5] event kills it.
One static term per statement is enough because [ENT-3]'s forward flow visits every statement of one function body exactly once: a loop body is walked once from the head state [ENT-5] forms before that walk, each `match` arm walks its own statements, and no statement is visited twice in one analysis.
A commit value therefore denotes that statement's value in the one abstract evaluation the walk performs, exactly as a counted header image denotes the binder's value in an arbitrary iteration, and every fact derived about it holds of each dynamic evaluation of that statement separately.

An FN-9 parameter datum denotes its function-entry image in the RelationTemplate but creates no snapshot term.
Local proof may reuse the ordinary parameter term only while FN-9's entry-image stability remains live; caller publication substitutes the corresponding pre-transfer actual image independently for each referenced formal.

A concrete goal is one finite typed expression tree with exact result `own Bool` formed under [FN-8]'s structural identity, either by concrete substitution of a GoalTemplate, by [ENT-3]'s goal-origin judgment in the current function, or as the canonical total predicate of an [ENT-6] operation obligation.
A concrete place datum retains the resolved root declaration event and its ordered field and `deref` projections; an actual substituted for a borrow formal uses the resolved referent datum, while an own actual uses its pre-transfer datum.
Named consts and typed literals retain the identities FN-8 fixes.

A direct value expression is the finite typed tree formed from those datums and the pure total operation rows admitted by [FN-8].
An admitted value expression is a finite tree recursively formed from direct-value rows and selected exact integer-operation or array-, buffer-, or slice-index rows.
Each selected partial row may enter that tree only after its own occurrence and every nested child obligation have succeeded in source evaluation order.
An index row retains its collection family and exact selected element, array-length, and slice-region arguments as applicable.
This admitted structure records the mathematical identity of the value already proved safe at that occurrence; it neither makes a subscript an L0 term nor authorizes evaluation before its owning nested obligation has succeeded.
Two occurrences of the same admitted typed tree therefore have the same value identity, but each occurrence separately discharges its nested operations and an earlier signed fact remains available only while [ENT-5] retains its support.

An evaluated-value datum is the finite occurrence-local identity for a value that has already been evaluated but has no admitted value expression.
FN-8's call-argument form is identified by `(concrete caller instance, call NodePath, argument ordinal, exact captured type, ordered projections, final result type)` and may occur only in the instantiated goal of that one ordinary call.
An [ENT-6] obligation-operand form is identified by `(concrete function instance, owning obligation NodePath, operand ordinal, exact captured type, ordered projections, final result type)` and may occur only in the canonical Goal queried for that one obligation.
Both forms are neither places nor L0 terms, have no direct or complete ordinary source goal origin, add no flow fact or place support, and cannot be established by naming or reevaluating their source expression.
Goal equality is exact typed tree equality, including every selected row and datum field, and therefore may hold across two source occurrences or concrete callee instances only when their complete typed trees are identical.
The finite goal universe of one concrete function is exactly the goals formed from its admitted Bool origins, requirement S4 sources, instantiated ordinary-call requirements, and the canonical OP-2 and OP-9 operation obligations, together with the finite parent and child trees their fixed decomposition and reconstruction rules visit.
Invariant targets and `proof_use` sources are affine inequalities rather than opaque Goals [INV-1, PRF-1]; an OP-4 bounds obligation remains an L0/affine relation and has no opaque Goal of its own.
Goal construction may intern only written subexpressions and the exact normalized components fixed by their owning rules; it synthesizes no arbitrary formula or unbounded algebraic search.

A signed opaque fact is exactly `+G` or `-G` for one concrete goal G, meaning that exact whole expression evaluated respectively true or false.
It carries no child facts merely by existing; [ENT-3] fact sources establish their selected signed contribution and [ENT-4] alone performs the finite parent reconstruction below.
If G's complete root is exactly one comparison origin relation R under [ENT-3], `+G` has the exact L0 projection R and `-G` has R's exact negation; a non-comparison root has no L0 projection.
The signed fact and its projection are distinct manifestations in one combined state and have the supports [ENT-5] fixes.

An atomic fact is one difference bound `t1 - t2 <= c` (t1, t2 terms, c a mathematical integer) or one disequality `t1 != t2`.
Difference-bound identity preserves the ordered term pair; disequality identity is the unordered endpoint pair, although the first source-normalization encounter preserves its written orientation for rendering and component order.
Source relations normalize exactly: `a <= b` is `a - b <= 0`; `a < b` is `a - b <= -1`; `a = b` is the bound pair `a - b <= 0` and `b - a <= 0`; `a >= b` and `a > b` swap operands; `a != b` is one disequality.
A constant operand folds through Z: `a <= 7` is `a - Z <= 7`.
Implicit facts hold at every program point: every term t carries the reflexive bound `t - t <= 0`; every term t of fragment type T carries `t - Z <= max(T)` and `Z - t <= -min(T)`; every measure term carries [MSR-2]'s standing facts; and every `len_of` or `cap_of` term over a place of type `array<T, N>` carries the equality to N (both bounds), with concrete N a constant and const-generic N a symbolic constant term.

[MSR-1] Four measure terms, over one place, for every measured value.
`len_of(P)`, `cap_of(P)`, `room_of(P)`, and `head_of(P)` are terms of the [ENT-2] term language, of fragment type u64, where P is an admitted measure place [ENT-2] clause (b).
An admitted measure place is a `place` [GRAM-5] formed with any number of field-selection `psuffix`es, `deref` wrappings, and subscripts, whose final selected type is a measured type.
The subscript admission is what makes `len_of(table[i])` a term, so a run of runs has provable operations; it is also why [MSR-2]'s granularity is stated over storage rather than over the word *element*.
Each such subscript is an [OP-4] occurrence like every other and owes that rule's own obligation against the base it indexes, submitted to [MSR-4] where the place is formed; a measure over a place whose subscripts are not all discharged is no term, exactly as an undischarged subscript in read position is no value.
An offset occurring inside a measure place is a written integer literal, a live `own` fragment-integer place, or an in-scope const generic [MSR-6], because the place's identity is decided over it: [OWN-7] decides two subscripted places by their offsets and [ENT-5] takes each offset's own support into every measure term the offset occurs in, so an offset neither relation can name would make two measures of two elements one term.
An offset of any other form in a measure place is not this rule's rejection: it is a place this version does not represent, reported as the compiler capability it is.
The same four spellings are the [OP-1] reader rows that read those quantities at run time: one quantity, one name, term and reader alike.

Which measures a type has, and whether each is *exact* or *bounded*, is table data.
The rule is that the table exists, gives every measured type a row, and gives every cell exactly one of *exact*, *bounded*, or *absent*.
An **exact** measure is one every writing operation publishes a value for; a **bounded** measure is one some writing operation can publish only a two-sided range for.
A measured type is exactly a type the table gives a row to; every other type has no measure term, and a measure former whose operand place has any other final selected type is the ordinary [TYPE-5] operand rejection at that place, carrying the measured types the table has a row for.
The table in this version is:

```wf-measures
| measured type            | len_of                   | cap_of             | room_of                | head_of                 |
|--------------------------|--------------------------|--------------------|------------------------|-------------------------|
| array<T, N>              | N, exact                 | N, exact           | 0, exact               | 0, exact                |
| buffer<T>                | allocated slots, exact   | len_of, exact      | 0, exact               | 0, exact                |
| Slice<'r, T>             | viewed elements, exact   | len_of, exact      | 0, exact               | 0, exact                |
| MutSlice<'r, T>          | viewed elements, exact   | len_of, exact      | 0, exact               | 0, exact                |
| FixedVector<T, n>        | initialized slots, exact | n, exact           | cap_of - len_of, exact | window origin, bounded  |
| Vector<'s, T>            | initialized slots, exact | slots taken, exact | cap_of - len_of, exact | window origin, bounded  |
| Arena<'s, bytes, align>  | cursor bytes, exact      | bytes, exact       | cap_of - len_of, exact | absent                  |
```

A `const` item of `FixedVector<T, n>` type reads that same row with `len_of` and `head_of` at their standing values `n` and `Z` rather than off a stored descriptor [CONST-2, S34]: the item is permanently read-only, so no [BLK-3] operation reaches it and no writing operation can move either quantity.
Exactly one cell class is *bounded* anywhere — a run's `head_of` — and it is the one cell the two run rows share: the two front-moving operations [BLK-3] publish it two-sidedly and no operation re-establishes it exactly, so no derivation may treat a run's window origin as a known constant after a front operation.
An `Arena`'s `len_of` is exact because every take from it requires `align >= align_ceiling(T)` [BLK-2], so its cursor is a multiple of `align` at every program point and the padding at a take is zero.
`Heap<'s>` has no row at all: a general store has no measure that means anything, and that is the absence of table data rather than an exception clause; a measure former over a `Heap` place is therefore the ordinary [TYPE-5] operand rejection this rule already states.
The table is data a later version extends with a row per measured type it adds; extending it adds no rule and amends none.

A measure is a logical quantity, and `head_of` is the origin of the logical coordinate system.
A measured value's initialized set is the `len_of` slots beginning at `head_of` taken modulo `cap_of`, and a **logical offset** `i` names the slot at physical offset `(head_of + i) mod cap_of`.
Every measure term and [OP-4] obligation is stated in logical coordinates, and one sentence carries a logical conclusion to a storage conclusion:

> `i |-> (head_of + i) mod cap_of` is injective on `[Z, len_of)` because `len_of <= cap_of`, so two disjoint logical ranges of one measured value describe disjoint storage.

That sentence is a definition proved from `len_of(P) <= cap_of(P)`, which [MSR-2] publishes as a standing fact; it is never a separate obligation an occurrence submits.
Where a row of the table gives `head_of` the exact value zero, the map is the identity and the sentence's conclusion is immediate; where a run's `head_of` is bounded and may be nonzero, the sentence is what carries two disjoint logical ranges of a wrapped window to two disjoint storage ranges, and it is the whole reason a logical obligation is a storage guarantee.

[MSR-2] Support is descriptor storage, a kill is an ordinary [ENT-5] event, and a standing fact has empty support.
A measured value's storage is two disjoint parts: its **descriptor storage**, the measure words its value carries, and its **element storage**.
The support of a measure term over P is P's descriptor storage, every borrow or content holder any prefix of P reads through, and the support of every offset occurring anywhere in P.
P's descriptor storage is exactly the [OWN-5] resolved place of P itself, not the resolved place of P's root: a measure of `frame.tail` is supported by `frame.tail` and not by `frame`.

The kill is [ENT-5]'s own rule with no new overlap notion: a measure term dies exactly on an [ENT-5] event whose written place overlaps its support under [OWN-7], where an event is any [SET-1] commit, [SET-2] commit, consume, scope exit, or any action carrying a `writes` occurrence that projects onto that storage under [EFF-2].
Stating the kill over the effect row keeps it closed when a later family derives a new action.
The granularity is stated once, over storage, and nothing is derived from the word *element*:

> A write at an element position of P overlaps the descriptor storage of `P[i]` and none of P's own descriptor storage.
> It therefore kills every measure of `P[i]` and no measure of P, whether the write is a [SET-1] commit, a [SET-2] `replace`, or an element write of a scalar — for which the set of killed measures is empty because a scalar has none.

Two consequences follow as derivations rather than clauses.
A write to a sibling field does not kill, because the descriptor storage of `deref(r).flags` and that of `deref(r).tail` do not overlap.
A write to an offset occurring in P kills at every level, because that offset's support is part of every enclosing measure term's support.
The element-position carve-out of [ENT-5] is removed rather than narrowed: it was true only because this version's measured types have no measured element type, which is a fact about the table and not a property of the word *element*.

At every point at which P is live these hold implicitly, as [ENT-2] implicit facts that no event kills:

```text
Z <= len_of(P)     Z <= room_of(P)     Z <= head_of(P)     len_of(P) <= cap_of(P)     head_of(P) <= cap_of(P)
```

The identity `len_of(P) + room_of(P) = cap_of(P)` is appended, as two inequalities, to [ENT-6]'s automatic affine-premise sequence, with the empty support every standing fact has.
The identity is a convenience for the writer and is never a route by which an operation's own post-state is derived, and a contract clause both of whose sides follow from these standing facts alone discharges no obligation.
A measure whose value the table fixes as a compile-time constant or a runtime-profile symbol is a standing fact with empty support: for `array<T, N>`, `buffer<T>`, and `Slice<'r, T>` both `room_of(P) = Z` and `head_of(P) = Z`, for `array<T, N>` both `len_of(P) = N` and `cap_of(P) = N`, for `buffer<T>` and `Slice<'r, T>` `cap_of(P) = len_of(P)`, and for `FixedVector<T, n>` `cap_of(P) = n`.
A row whose cell is *bounded* fixes no such constant: a run's `head_of` is a standing fact only through `Z <= head_of(P)` and `head_of(P) <= cap_of(P)` above, and a run's `len_of` and `room_of` are ordinary killable terms.
A standing fact holds at every program point of P's scope and no event kills it, exactly as an [ENT-2] implicit fact does.

[MSR-3] One denotation per operand position, keyed on the parameter's mode and the explicit entry former.
The complete measure table is:

```text
| measure operand position                         | inside the callee     | at the caller          |
|--------------------------------------------------|-----------------------|------------------------|
| requires, any parameter                          | entry image           | pre-transfer term      |
| ensures, own parameter                           | immutable entry datum | immutable call datum   |
| ensures, shared-borrow parameter                 | immutable entry datum | live term              |
| ensures, bare &uniq parameter                    | exit-state term       | resolved exit place    |
| ensures, entry(&uniq parameter)                  | immutable entry datum | immutable call datum   |
| ensures, result binder                           | selected result       | result destination     |
```

The proof-only former `entry(parameter)` is admitted only in an `ensures_clause` and only when its direct IDENT resolves to a `&uniq` parameter of that function; every other occurrence is a hard error citing MSR-3 at the former, with the restructuring `use entry only on an exclusive parameter in ensures`.
It has that parameter's ordinary borrow type for projection checking. Ordinary explicit dereference and field projections follow it, as in `len_of(deref(entry(heap)))` and `len_of(deref(entry(frame)).tail)`; it is no runtime value, allocation, holder, or snapshot copy.
The former selects the entry denotation of the projected measure. A bare `len_of(deref(heap))` in ensures instead selects exit state. A nested `entry`, an expression argument, and entry of a local or of an own or shared parameter are not admitted.
Non-measure parameter datums retain [FN-9]'s entry-image stability judgment; this former adds no scalar snapshot family.
An `own` operand denotes the call datum because its caller cannot name the consumed value's post-state. An exclusive referent is still the caller's resolved place after the call: its exit measures can therefore be checked at returns and instantiated there without transferring its owner.
Entry and exit measures are distinct terms even when both project from the same formal and actual. The exact projected effects kill the caller's supported facts before the verified exit relations establish [CALL-6]; no syntactic property of an actual may retain or kill a fact in place of that effect judgment.
Kernel records use this same spelling, explicit dereference, and denotation, with no separate snapshot notation [BLK-0].

A **call datum** is a compiler-owned immutable [ENT-2] term with empty support: no place occurs in it, no [ENT-5] event kills it, and no later write retargets it.
There is one former, keyed on what a datum denotes: a datum is identified by `(that call's NodePath, the formal ordinal, that operand's ordered projections, whether it denotes the operand's value or its length)`, is compiler-owned and immutable, and is established equal to that operand's pre-transfer term at the call's pre-transfer point [ENT-3.S13].
A datum is formed, never proved.
When the operand's pre-transfer term is itself immutable with empty support — a constant, a symbolic const-generic parameter, a counted capture, a commit value, or another call datum — nothing can retarget what that term denotes, so the datum is that term and no second one is formed; every other operand mints its own.
Its placement is the call, which is one of the events at which the language undertakes to carry a value's measures.

An **entry datum** is the same former at the second placement, body entry.
For each parameter of measured type and each [MSR-1] measure of it that a declared relation of that function names, one compiler-owned immutable term is identified by `(the formal ordinal, that operand's ordered projections, which measure it denotes)` and established equal to that measure at body entry.
It is the same kind of term as a call datum and carries the same closure: no place occurs in it, no [ENT-5] event kills it, and no later write retargets it.
That is what the immutable entry-datum cells of the table above denote.
A body that replaces an `own` parameter's local binding with newly constructed storage — `let old = replace vector = move fresh;` — therefore leaves every clause naming that parameter's measure meaning exactly what it read as at entry, and the caller reading the same clause after substitution reads that call's call datum, which the same statement's consume cannot kill either.
For a `&uniq` parameter the immutable entry datum is named explicitly through `entry(parameter)`; the bare parameter's measure instead denotes the selected return's resolved referent.
An entry datum is formed, never proved, and it is not a second fact source: its standing orderings [MSR-2] reach it through the equality it is established with, exactly as they reach any other term.
A parameter operand that is not a measure keeps the entry-image judgment [FN-9] states over the live place, since a value of fragment type is not a measured value and has no measure datum.
A **placement datum** is the same former at every remaining placement, each of which is one naming event inside a body at which a measured value crosses from one place to another.
For each such event, one compiler-owned immutable term per [MSR-1] measure is identified by `(that statement's NodePath, which placement of the table below it stands at, the ordinal within that statement, which measure it denotes)`, is established equal to that measure of the event's **source** place immediately before the statement's own kills, and is established equal to that measure of the event's **destination** place at the statement's normal continuation.
It is the same kind of term as a call datum and carries the same closure: no place occurs in it, no [ENT-5] event kills it, and no later write retargets it.
That the datum is minted before the statement's kills and read after them is the whole content of every placement: the event consumes or overwrites the source, so without a term with empty support standing between the two places a measured value would arrive at its new place with no measures at all, and `let built = move spare;` would lose what the caller proved about `spare`.
A placement datum is formed, never proved, and it is not a second fact source: its standing orderings [MSR-2] reach it through the equality it is established with, exactly as they reach any other term.
The complete placement table is:

```text
| the naming event                                                      | source place                | destination place            |
|-----------------------------------------------------------------------|-----------------------------|------------------------------|
| the REBIND: one `let` binder, or one [LIV-2] `set` target that is a    | that place                  | the binder, or the place the |
|   place, whose right-hand side at that ordinal is a bare use of one    |                             |   commit writes              |
|   measured place                                                      |                             |                              |
| the ELEMENT: the same, where the [LIV-2] target is an element         | that place                  | `P[i]`, the element position |
|   position of a run                                                   |                             |   the commit writes          |
| the DISPLACED value of one [SET-2] `replace`                          | the place the `replace`     | the binder that receives the |
|                                                                       |   writes, before it writes  |   displaced value            |
| the CONSTRUCT: one field operand of a constructor `call` that is a bare use  | that place                  | that field of the            |
|   of one measured place                                               |                             |   constructed value          |
| the DESTRUCTURING: one binder of a destructuring consume [GRAM-4]     | that field of the operand   | the binder                   |
|   whose operand is a bare use of one measured nominal place           |                             |                              |
| the PAYLOAD: one arm binder of a `match` whose scrutinee is a bare    | that field of the           | the arm binder               |
|   use of one enum place                                               |   scrutinee's payload       |                              |
```

A right-hand side, operand, or scrutinee that is anything but a bare use of a measured place mints none, and the ordinary sources establish whatever that expression publishes.
The element and displaced placements name an element position, and an element position is a place exactly where its offset is one a place relation can name [MSR-1]: a written literal, a live `own` fragment-integer binding, or an in-scope const generic [MSR-6].
Two element places are decided by their offsets [OWN-7], so an offset provably distinct from nothing — itself included — would relate two elements of one run as one term; a commit at such an offset carries no measure and, being an element write of unknown position, kills every measure of every element of that run [MSR-2].
The element placement reaches only a written element position, so a boundary operation [BLK-3] carries no measure through the slot it writes: `place_back` stores its value at the entry length of its referent and `take_back` takes one from the exit length of its referent, and a measure term is not an offset this version admits.
A run put into a slot by a boundary row and taken back out by one therefore arrives with no measures of its own, and a caller that needs one reads it and branches [MSR-4].
The payload placement names a field of an enum place's payload, and a tracked place's path is field selections, `deref` wrappings and subscripts [ENT-2] — none of which names a variant.
It is therefore stated over a nominal enum exactly one of whose variants carries fields, where the field path selects one storage on every execution; the prelude `Option` is such a nominal and the prelude `Result` is not, its `Ok(value)` and `Err(error)` being two storages one path cannot separate.
DEFERRED: the payload placement over an enum more than one of whose variants carries fields, which needs a place step that names the variant it selects; its delta is numbered rules +0 and grammar productions +0.

*Judgment:* the denotation at every operand position, and the restriction of `entry(parameter)` to an exclusive parameter in `ensures`.
*Publishes:* the call datum at the call placement, the entry datum at the entry placement, the placement datum at every placement of the table above, and the denotation table.

[CALL-1] Through a shared borrow, every fact survives.
For an argument whose declared parameter mode is `&'r`, of any type, run and view included, the call is a kill event for no fact supported by that actual's resolved place.
The whole ground is [OWN-5]'s shared-holder prohibition: no write through a shared holder is admissible, so a body can exhibit none and [EFF-2]'s both-ways check admits none in the declared row, so no `writes` occurrence projects onto that place and no [MSR-2] kill fires.
That ground is exactly as strong as the set of actions this document classifies as writes, so a later family that makes a new action a write of a borrowed referent reaches this rule through [EFF-2] alone and needs no clause here.

*Judgment:* none; the absence of a kill, which is [MSR-2]'s kill not firing.
*Publishes:* the survival of every fact supported by a shared-borrow actual's place.

[CALL-2] Through a value passed and returned, only the contract's facts exist on the result.
An `own` argument whose type is affine or linear [OWN-1, PROV-6] is a consuming use, so every fact whose support contains that binding's root dies at the call [ENT-5](c), that place's measures included.
An `own` argument whose type is copy is a duplicate and not a consuming use, so the caller's place and every fact supported by it survive the call and are available at the next one.
The result is a fresh binding carrying exactly the callee's declared relations, established as [CALL-6] states, and no fact of any argument reaches it: a caller that wants a relation between what it handed over and what it got back reads it from the contract or does not have it.
A declared relation may name a consumed operand's measure, which at the caller denotes that call's call datum [MSR-3]; a datum has empty support, so the consume the same statement performs cannot kill it and the relation means what it reads as.

*Judgment:* the ordinary [ENT-3.S12] establishment, subject to the denotation [MSR-3] fixes.
*Publishes:* the callee's declared relations on the result, and nothing else on it.

[CALL-3] A write through a view reaches the range's storage and no measure of the origin place itself.
For a parameter of loan-bearing type [VIEW-1], own or behind a borrow, a projected callee `writes` occurrence kills every fact whose support overlaps the **viewed range's storage**, which for an element type having descriptor storage of its own includes that element's measures, and kills **no measure term over the origin place itself and none over the view**.
For every other parameter the projected write kills measures as an ordinary descriptor-storage-overlapping [ENT-5] event [MSR-2].
The classification is stated over storage and nothing is derived from the word *element*, exactly as [MSR-2]'s granularity is: when the viewed element type is itself measured, the viewed range's storage **is** the descriptor storage of the origin's elements, so a measure of a viewed element dies and a measure of the origin survives.
The descriptor/element split is a property of the element type, not of the word *element*.
This judgment applies to every admitted viewed element type [TYPE-2, VIEW-2], including one whose element storage contains a descriptor.

*Judgment:* the kill classification per declared parameter, which is [MSR-2]'s judgment parameterized by what the transport reaches.
*Publishes:* the surviving measures of the origin place and of the view.

[CALL-5] No transport reads the actual's spelling.
The transport a call selects for one argument is fixed by the callee's declared parameter mode and type and by its declared contract, and by nothing else: not the argument expression's shape, not the callee's body, not its name, and not any per-parameter summary derived from a body.
An ordinary function's declaration is its complete call boundary whether its definition is a Whitefoot body or supplied by linking. The kernel-domain rows [BLK-0] likewise use only their stated declaration and transport rules.
The exact declared effect row is projected onto each actual's resolved places. A caller fact dies exactly when a projected write overlaps its ordinary support [ENT-5, MSR-2]; the actual's syntactic shape creates no write and removes none. A parameter absent from the declared writes has no write kill.
For a projected write for which no view transport is selected, the affected extent is the ordinary descriptor storage [CALL-3]. Thus a whole-run write through a `&uniq` formal kills the old window facts, including when the actual is a holder or a nested field. A body that changes only elements can still have the same declared `writes(run)` row as a whole replacement; callers frame neither body beyond what that exact declaration states.
Run mutation uses the exclusive boundary rows [BLK-3]; a source helper over an exclusive run parameter may publish the verified exit measures it promises [FN-9, MSR-3]. With no ensures the caller obtains no replacement fact from the mere presence of an exclusive parameter. A genuine empty/nonempty branch after rereading length may supply a new fact; no runtime check substitutes for a required static proof.

*Judgment:* the conservative default for every parameter no transport is selected for.
*Publishes:* the absence of any call-site-derived or body-derived classification.
*Amends:* [ENT-5]'s clause (b), whose projected-callee-write kill is now classified by [CALL-1] through [CALL-3] and by nothing else.

[ENT-3] The fact state is defined constructively over the conservative structural normal-control graph [FN-1]: each source below establishes its L0 and signed-goal facts at its stated point; facts flow forward along normal edges; kill events apply on the edges where [ENT-5] places them, with scope-exit kills applied before any join; merge points take the [ENT-5] join and loop heads the [ENT-5] loop rule; and the state queried at any point is the [ENT-4] closure of that flow.
retired: S8, S10
Dominated straight-line establishment is a consequence of this construction, not a second definition.
Nothing else is a fact: a writer's `ensures_clause` is only an FN-9 proof obligation, never a trusted source; a written header or local invariant conclusion has no authority until INV-1 and any applicable PRF-1 certificate prove it; no struct invariant, compiler-invented loop proposition, inferred summary, or unverified user-function result exists.
S11 is only the compiler-owned consequence of the counted operations [FN-1] actually executes, and S12 exists only from the declaration relations available under FN-9: a separately verified earlier-SCC summary or a PRE-1 supplied declaration, under the publication formula below.
Each accepted fact retains the constructor identity and direct parents that already produced it; this diagnostic information establishes and kills no additional relation or signed goal, and no [ENT-4] answer depends on a second provenance state.

A comparison origin is defined first.
An expression has comparison origin R when (a) it is an `infix` expression whose operator is a `compare_op` — `==`, `!=`, `<`, `<=`, `>`, `>=` [OP-2] — and whose two operands are each a term or constant, R the corresponding relation over them; or (b) it is a bare IDENT naming a `let` binding of type `own Bool` whose initializer right-hand side satisfies (a) with relation R, no [ENT-5] kill event (a)–(d) applies to a fact supported by an operand term of R on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`, user-function results, and deeper indirection chains contribute no L0 comparison origin in this version; an established Boolean goal contributes relations only through the members of its signed decomposition set.

An expression has integer-domain-predicate origin G when (a) it is one total `+defined`, `-defined`, `*defined`, `/defined`, `%defined`, `ineg.defined`, `iabs.defined`, `ishl.defined`, or `ishr.defined` operation with its selected concrete operand type and complete ordered admitted value-expression identities, after every nested obligation in those operands has succeeded, G that exact typed GoalExpression; or (b) it is a bare IDENT naming an own-Bool ordinary-let binding whose initializer satisfies (a), no [ENT-5] kill event applies to G's support on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
This origin is one ordinary exact goal, not a second fact channel.
Its support, expansion, kills, scope exit, joins, and signed establishment are the ordinary goal rules below.

A Bool expression has an ordinary goal origin G when, after its ordinary expression judgment and every nested operation obligation have succeeded, its completely typed expression is one admitted value expression and its root is a pure total operation-table row, with exact tree identity as [FN-8] fixes.
Construction, an ordinary function call, a move or borrow, an undischarged partial operation, an expression requiring occurrence-local evaluated-value identity, and every other expression shape has no goal origin.
A checked exact integer operation or subscript may therefore occur only below that total root and only through the admitted structure above; it never establishes its own safety merely by occurring in G.
The unexpanded tree G is the direct goal.
Starting from that direct goal, its complete origin expansion recursively replaces an ordinary-let datum by that binding's unique defining right-hand side exactly when the right-hand side itself has an admitted value expression formed after its own nested obligations succeeded, the binding is no `set` target on any path from that initializer to this use, and no [ENT-5] kill event applies to the replacement's support on any such path.
Expansion continues to a fixed point and is all-or-nothing for every eligible leaf; it never performs an algebraic rewrite.
The goal-origin set is the direct goal plus that one complete valid expansion when it differs.
Thus a condition binding's own Bool value and its still-valid computation origin are both retained: a later write to an origin place kills the expanded goal but not the already-computed binding goal, while a write to the binding kills the latter normally.
Definition expansion in FN-8 is unconditional because every `contract_define` is erased pure proof syntax and the admitted block contains no mutation.

Signed Boolean decomposition applies at every ordinary establishment of a signed goal fact by the sources below.
The decomposition set of `+G` whose complete root is `band(A, B)` is `+A` and `+B` together with each member's own decomposition set; the decomposition set of `-G` whose complete root is `bor(A, B)` is `-A` and `-B` together with each member's own decomposition set; the decomposition set of either sign of `bnot(A)` is the opposite sign of A and its recursive decomposition; `-band` and `+bor` remain exact disjunctive roots and have no child member.
Every admitted member whose root has an exact comparison projection also establishes that signed projection.
Each member has its own [ENT-5] support, kills, joins, and loop treatment.
This is a finite structural walk with no algebraic rewrite.

The sources are:

[ENT-3.S1]
- S1 (branch facts).
At an `if_stmt` or `value_if`, each goal G in the condition's goal-origin set is established as `+G` at the then-block's entry and `-G` at the else-block's entry; for an else-free `if_stmt`, `-G` is established on the false edge, which joins the then exit at the continuation [ENT-5].
Independently, when the condition has comparison origin R, R is established at the then entry and R's exact negation at the else entry or false edge.
L0 negation is exact over mathematical integers: the negation of `a - b <= c` is `b - a <= -c - 1`; the negation of `a = b` is `a != b` and conversely.
[ENT-3.S4]
- S4 (requires facts).
At a concrete function-body entry, its complete instantiated [FN-8] goal G is established as `+G`.
When and only when G's complete root is one comparison admitted by comparison-origin shape (a), whose operands after template and call substitution are each an admitted term, constant, or `len_of(P)` length term, that exact relation R is also established.
Beyond that projection, only the members of G's signed decomposition set and their projections are established; no other child of any goal is established.
S4 is the admitted-body axiom justified by every ordinary caller's static discharge; no callee-entry prologue or boundary check executes.
[ENT-3.S5]
- S5 (copy and conversion equalities).
An `ordinary_let_rhs` establishes at its binding: for `let x = lit;`, x = value(lit); for `let x = p;` with p a term of type T, x = p; for `let y = cvt::<Src, Dst>(p);` with (Src, Dst) a total pair [OP-6] and p a term or constant, y = p — `cvt` keeps its written type pair [TYPE-5].
A successful [SET-1] commit to a direct fragment-typed place first evaluates its right-hand side to that occurrence's commit value v, establishing at v exactly the [ENT-3] image the same right-hand side establishes at an `ordinary_let_rhs` binding: this clause's three rows and every S6, S7, and S9 row whose conclusion is a relation over the bound value itself.
A row concluding instead over a length term of the destination place has no commit form, a commit value being no place.
Every fact supported by the old target value then dies under [ENT-5], and only then is the post-write equality x = v established.
Evaluating v before that kill is what lets [ENT-5]'s pre-kill closure carry the value's surviving consequences across the write, and the equality still carries no old target fact, since v is a term distinct from x and every fact naming x has died.
An array- or buffer-index target and a non-fragment target receive no commit value, and a right-hand side whose form matches no image row forms none either: with no commit value to name, S5 establishes no post-write equality and adds nothing to the state [ENT-5]'s kill leaves, and no S5 commit image beyond that exists in this version.
[ENT-3.S6]
- S6 (length facts).
`let b = buffer_new(n, v);` and `let b = buffer_vacant::<T>(n);` each establish len_of(b) = n on the normal continuation [OP-9], n read as term or constant.
`let m = len_of(P);` for a tracked P establishes m = len_of(P).
`let s = slice_of…(&P);` for a tracked P establishes len_of(s) = len_of(P).
[ENT-3.S7]
- S7 (constant-offset arithmetic).
For `let s = p +wrap k;` with p a term of type T and k a constant in either operand position, when the closed state at that point derives `min(T) <= p + k` and `p + k <= max(T)` (as bounds on p through Z), s = p + k is established; `p -wrap k` with constant k establishes s = p - k under the dual range condition.
For proof-required exact `p + k` and `p - k` with constant k, s = p ± k is established on the normal continuation unconditionally after source acceptance: that exact site's discharged IntegerDomain obligation is the proof [OP-2, ENT-6].
For a `match` whose scrutinee is directly `p +checked k` or `p -checked k` with constant k, or a bare IDENT let-bound to one where no [ENT-5] kill event applies to a fact supported by p between the initializer and the match and that binding is no `set` target on that path, the `Ok(value: w)` arm establishes w = p ± k at arm entry; the `Err` arm establishes nothing.
For a direct ordinary binding `let q = a / k;` at an unsigned integer type, when a is an admitted [ENT-2] term or constant, k is a positive written integer literal, and the exact division's ordinary IntegerDomain obligation has succeeded, establish the L0 relation `q <= a` and retain the separate affine value image `k*q <= a` over the exact current value images of q and a.
The L0 relation is an ordinary S7 fact; the scaled relation is one specification-fixed member of [ENT-6]'s automatic affine-premise list and is not copied into L0.
Replacing q or a creates a new value image and cannot retarget either relation to the replacement; a still-live alias of an old value may continue to use the old relation under [ENT-5].
A signed division, a nonterm dividend, a nonliteral divisor, a zero divisor, a result not introduced by the direct binding, and every other division form establish neither relation.
For a direct ordinary binding `let r = a % d;` whose exact remainder IntegerDomain obligation has succeeded, unsigned r establishes `r < d` when d is an admitted term or constant, while signed r establishes `-(|d|-1) <= r` and `r <= |d|-1` only when d is a nonzero written integer literal or earlier named integer const whose absolute value and endpoints are representable in the proof domain.
No other remainder form establishes those relations.
Additionally, for a direct ordinary binding `let r = iand(a, b);` at unsigned integer type T, establish `r <= a` when a is an admitted term or constant and independently establish `r <= b` when b is one, in operand order; signed `iand`, every other bit operation, a nonterm operand, and a result not introduced by that direct binding establish no such relation.
For a direct ordinary binding `let r = ishl.wrap(one, count);` at unsigned integer type T, establish `r != Z` exactly when `one` is directly a checked typed literal or directly an earlier named const whose mathematical value is one.
A local binding merely proved equal to one, a const-generic value equal to one, a signed result, any other left operand, a non-direct result, and every other shift mode establish no nonzero fact.
The latter is sound because [OP-8] masks count modulo T's width, so shifting the one bit never clears it.
[ENT-3.S9]
- S9 (const-array element ranges).
For `let x = c[i];` where c is the bare IDENT of a named const of type `array<T, N>` [CONST-2] and T a fragment type, with vlo and vhi the minimum and maximum of its N declared element values, vlo <= x and x <= vhi are established at the binding.
The index's own bounds obligation [ENT-6] is judged separately and is unaffected.
Deeper const shapes establish nothing in this version.
[ENT-3.S11]
- S11 (counted-range structural facts).
In a `for_stmt` preheader, immediately after the lower and upper endpoint values have been captured once in [FN-1]'s order, establish `lower_capture = lower_endpoint` and `upper_capture = upper_endpoint`, reading each admitted endpoint as its exact term or constant, and establish `binder = lower_capture` at the compiler-owned initialization.
Close that post-capture state under [ENT-4] before [ENT-5] forms the counted head state.
On every true header edge that actually enters the body, establish `lower_capture <= binder` and `binder < upper_capture`; the first follows from initialization plus the exact representable compiler updates, and the second from the header comparison just executed.
The capture-to-endpoint equalities are established once in the preheader only: no later header or body entry rereads an endpoint or reasserts a capture equal to the current value of a mutable endpoint source.
The false header edge, every `break` edge, and the counted continuation establish no S11 fact and in particular no raw `binder = upper_capture` postcondition.
Before the binder and captures leave scope, [INV-1]'s separately proved exact-exhaustion rule may use the false guard to form a binder-free affine conclusion; that conclusion is an INV-1 fact, not S11 and not a retained binder equality.
[ENT-3.S12]
- S12 (ordinary declared normal results).
S12 has one owning `CallResultPublication(c,q)` judgment in the ordinary semantic flow.
That judgment succeeds only when q belongs to a declaration relation available under FN-9, either supplied by PRE-1 or atomically published by a strictly earlier call-graph component; every actual-expression obligation and instantiated FN-8 requirement of c is discharged in the caller before transfer; every referenced formal has its exact pre-transfer substitution; ordinary consumes, borrow commits, projected effects, writes, target commits, and kills have run in the fixed order below; and every support of the substituted result relation remains live.
The candidate relation and all of those parent derivations remain private until every source-semantic judgment in the compilation unit succeeds, then enter the checked program in the same failure-atomic publication as their call and function.
Failure of any premise or any later source-semantic judgment discards the candidate and the complete prospective checked program.
This is the original construction of S12, not a second provenance pass or a check of compiler-generated data.
For one call c and verified relation q, use exactly FN-9's `A0(c)` and per-relation `M(c,q)`.
Candidate scratch establishes q once in the current ProofContext exactly as [FN-9] fixes, after ordinary transfer and every applicable consume, borrow, callee-effect, and target kill.
Each substituted formal is independent: a referenced actual that has no ENT-2 image makes only that q unavailable, while an unreferenced non-ENT-2 actual has no effect on q.
Occurrence-local call-argument evaluated-value datums never enter q.
For relations that name a result, the only result destinations are the fresh direct ordinary-let binding, direct-call selected `Ok` payload, the direct-set target place — of which [FN-9]'s narrow direct-set receiver is the case where that same target is also an argument — narrow selected-payload outer receiver, each binder of a destructuring `let`, and each target of a `set` target list [FN-9, CALL-4, ENT-5].
The last two take result ordinal i at binder or target i and exist only for a declaration that writes an ordered result list [GRAM-2, GRAM-4]. An unrouted result-free relation over exclusive exit state needs no result destination: it establishes once on the call's normal continuation at its resolved exit places, after the same kills, including for an expression statement and a unit call.
A named or pending outcome, stored or propagated whole outcome, false matching predicate, killed support, or rejected call establishes nothing.
The complete candidate set stays unchanged in failure-atomic scratch until the owning source judgment succeeds; any failure publishes none, and success commits all of them atomically.

[ENT-3.S13]
- S13 (call datums).
At an ordinary source call whose callee has an atomically published summary, each `own` operand and each explicitly entry-qualified `&uniq` measure of each declared relation of the resolved callee mints one call datum [MSR-3] and establishes it equal to that operand's exact pre-transfer term, at the pre-transfer point of [ENT-5]'s call-boundary order and before that boundary's consumes, borrow commits, callee-effect kills, and target kills.
The population of this source is every callee whose declared relation list is published data: an ordinary function with its FN-9-verified or PRE-1-supplied contract and every kernel-domain row [BLK-0]. A PRE-1 signature and a kernel record are declaration data and require no source-body earlier-component verification premise; neither gains an additional result-fact source.
The same datum formation applies to source summaries and kernel records. An exclusive exit measure is never a call datum: it is the ordinary live term after the call's exact projected effects and the statement's own kills.
The operand's pre-transfer term is the one [FN-9]'s `A0(c)` substitution already fixes; the datum adds no term the substitution could not name and no relation the callee did not declare.
A datum has empty support, so [ENT-5]'s pre-kill closure carries its consequences across the same statement's kills while every fact whose support those kills remove dies normally.
An operand the substitution leaves without an [ENT-2] term mints no datum, exactly as it makes only that relation unavailable under `M(c,q)`.
S13 is the one source by which a declared relation's substitution is computed at a point other than the point at which the relation is established, and [CALL-6] states both points.

[ENT-3.S14]
- S14 (admitted product interval).
At an `ordinary_let_rhs` binding or a [SET-1] commit whose right-hand side is one non-constant integer multiplication that [ENT-6]'s fixed interval-product rule admitted, establish on the bound value the two constant bounds that rule's four endpoint products fix: the least of those four products is no greater than the bound value, and the bound value is no greater than the greatest of them.
The published bounds are exactly the measurement the domain decision consumed, so an implementation states them from that one computation and never proves the endpoints a second time; a domain discharged by the finite L0 route or by an affine clause publishes nothing here, because neither computed those products.
Both relations name only the bound value and the distinguished zero term Z, so the support [ENT-5] derives is the bound value alone.
That is what they mean: they describe the value the multiplication already produced, so a later write to either operand leaves them true, while a write to the bound place kills them under the ordinary rule.
A `let` binder is fresh and a commit value is compiler-owned [ENT-2], so the bound value never aliases an operand.
This source adds no relation over the operands, no term the multiplication did not already bind, and no route by which a product enters an automatic premise family: a written `use` remains the only way a product participates in a certificate.

The label S8 is retired, not reused: its midpoint family was struck as an owner-approved version amendment and may return as a later version's monotone addition the day a corpus program writes the shape.

[CALL-6] Publication: how a declared relation becomes a fact, where it is computed, where it is established, and that the set it belongs to is consistent.
Every published relation in this document is published by exactly one route — [ENT-3.S12]'s, with [ENT-3.S13]'s substitution — and nothing else publishes anything.
This rule states that route's four points once, so no rule computes a fact at one program point and uses it at another without naming both.

A declared relation is **instantiated at the call**, by substituting each operand at the denotation [MSR-3]'s table gives it: an `own` measure or an explicitly entry-qualified exclusive measure by that call's pre-transfer datum [ENT-3.S13], a shared-borrow measure by its live resolved referent, a bare exclusive measure by the actual's resolved exit place, and a referenced result binder by its destination below. Entry and exit terms are distinct even when they name one formal.
Its **support** is the ordinary L0 support of the substituted terms. The immutable call datums have empty support; an exclusive exit term has the support of the resolved place after the call's projected write kills. Those writes kill pre-call facts, not the exit relation that the verified callee establishes afterwards. A later target commit or other write to that place kills the exit relation normally.
It is **established** on the call's normal continuation, after the call's ordinary transfer, consumes, borrow commits, target commit and kills, exactly in [ENT-5]'s call-boundary order.
A relation routed to a variant is instantiated at the call in the same order and is **restricted** to that variant's arm: it is available exactly on the paths on which that arm is entered, and it is not deferred to the arm, so an [ENT-5] event lying between the call and the arm kills a relation whose support it removes rather than preceding an establishment that has not happened.
A relation whose support is dead is not available at all; a relation over a call datum has empty support and no event kills it.
A relation naming results uses exactly [ENT-3.S12]'s closed result-destination list [CALL-4]. An unrouted relation naming only exclusive exit state is established on the ordinary normal continuation even when no result is bound; its destination is that resolved state, and a unit return adds no result datum.

Every published relation set is checked for consistency at the declaration.
A `contract_block` whose instantiated relations are contradictory at their establishment point is a hard error citing CALL-6 at the `fn_decl`, `ContradictoryPublishedRelations`, naming the clauses and carrying the restructuring `state one consistent relation set: a contract whose clauses cannot hold together publishes every fact at every caller`.
The set is partitioned by route first, because a routed clause is available only on its own arm and two clauses on two arms are never in one caller state together; an unrouted clause selects every explicit return [FN-9] and is therefore a member of every route's set.
Contradiction is the ordinary [ENT-4] question over the declared templates: each distinct operand datum is one term, a literal folds through Z with its value, and the set is contradictory exactly when its transitive closure derives a negative self-bound or forces two terms one declared disequality separates to be equal.
A template whose operand shape that closure cannot represent contributes no premise, so a reported contradiction is always a real one.
The judgment is at the declaration because the set is fixed there: at a contradictory point every L0 relation and both signs of every goal are derivable [ENT-4], so an inconsistent contract is not one wrong fact at a caller but every fact at every caller, and no caller state repairs it.
A contradictory `requires` set is a different thing and stays admissible: it makes the instance legally uninhabited [FN-8], publishes no relation, and no reachable non-contradictory caller can call it.

*Judgment:* the S13 instantiation at the call, the establishment and restriction, the kill from the call, and the consistency check at the declaration.
*Publishes:* the source, the substitution, the instantiation point, the establishment point, the destination list, and the support of every declared relation in the language.

[ENT-4] The L0 component of the closed fact state is the least set containing its established and implicit facts and closed under exactly: (1) from `t1 - t2 <= c1` and `t2 - t3 <= c2`, derive `t1 - t3 <= c1 + c2`; (2) from `t1 - t2 <= 0` and a disequality between t1 and t2 in either orientation, derive `t1 - t2 <= -1`; (3) of two bounds on one ordered pair, the smaller constant subsumes.
L0 derivability is exact: `a - b <= c` is derivable when the closed state contains `a - b <= c'` with c' <= c; `a = b` when both `a - b <= 0` and `b - a <= 0` are derivable; `a != b` when a disequality is present or `a - b <= -1` or `b - a <= -1` is derivable.

The opaque component retains established signed facts and the following finite truth-functional parent reconstruction over exact parent goals already interned in [ENT-2]'s universe.
`+band(A,B)` derives from both `+A` and `+B`; `-band(A,B)` derives from either `-A` or `-B`; `+bor(A,B)` derives from either `+A` or `+B`; `-bor(A,B)` derives from both `-A` and `-B`; and either sign of `bnot(A)` derives from the opposite sign of A.
Literal `True()` has an implicit positive proof and literal `False()` an implicit negative proof.
No `bxor` or Boolean-equivalence introduction is admitted in this version.
The closure considers only already-interned exact parent trees, uses the written rule order and minimum non-cyclic derivation depth, and creates no new formula.
Exact signed-goal identity includes every selected operation-table row, concrete selected operand type, and complete ordered operand GoalExpression.
`+G` is derivable when that exact positive fact is present, when G has an exact comparison projection R and L0 derives R, when G is an integer-domain predicate whose fixed [ENT-6] component normalization proves true, or when G's comparison root has an affine normalization and `AUTO` proves it. The affine route is the goal's own comparison normalized, so proving it proves the goal and an L0 projection is what the retained evidence names rather than what the route requires: a goal carrying a coefficient has no two-term projection to name and its retained derivation is the affine consequence alone.
`-G` is derivable when that exact negative fact is present, when G has a comparison projection and L0 derives R's exact negation, or when G is an integer-domain predicate whose fixed normalization proves false.
Integer-domain component relations are only an alternate derivation route into that same exact signed goal; they establish no second source goal and receive no source-obligation identity of their own.
Derivability never decomposes a merely derived parent: [ENT-3] decomposes only the specification-enumerated source establishments.
One retained proof never uses a parent-to-child source derivation and then that child solely to reconstruct the same parent; deterministic minimum-depth selection therefore contains no parent-child-parent cycle.

The combined state is contradictory when L0 derives `t - t <= -1` for any t or when both signs of one exact goal are derivable.
At a contradictory point every L0 relation and both signs of every goal in the finite universe are derivable and every ordinary obligation, call goal, and FN-9 selected-return relation is discharged.
At a non-contradictory query point, an instantiated goal G is `discharged` when `+G` is derivable, `refuted` when `+G` is absent and `-G` is derivable, and `unproved` otherwise.
An instantiated L0 relation R is `discharged` when every normalized conjunct of R is derivable, `refuted` when R is not discharged and R's exact negation is derivable, and `unproved` otherwise.
A one-bound negation is S1's reversed strict bound, an equality relation's negation is its disequality, and a disequality's negation is the equality's two-bound relation.
These three dispositions are complete and exclusive [FN-8, FN-9].
The least closure is unique and finite up to L0 subsumption because only the finite terms and goals [ENT-2] participate and the rules are monotone.
Implementations may compute lazily or incrementally, but every derivability and disposition answer must equal this least-closure answer.

[ENT-5] The support of an L0 fact is every tracked place occurring in its terms; every compiler-owned counted capture term occurring in its terms; for each measure term over P, P's descriptor storage and the support of every offset occurring in P, but not P's element storage [MSR-2]; and every borrow or box/arena holder binding any of its places reads through by `deref`, a bound call-result holder included — its resolved place is the candidate actual's complete resolved place [OWN-6], so a `set` commit or projected callee write through the chain kills exactly the facts supported by that storage.
Z, literals, named const values, and every measure datum of [MSR-3] — a call datum, an entry datum, and a placement datum alike — have empty support and never die.
A counted capture is immutable and can die only on an edge leaving its compiler-owned construct scope.

The support of either sign of an opaque goal is the union of the resolved places whose values its complete typed expression reads.
A direct binding goal therefore depends on that binding, while its separately established complete origin expansion depends on the places read by the expansion.
For a measure node over P, support includes P's descriptor storage, every holder used to reach it, and the support of every offset occurring in P, but not P's element storage, under the same descriptor-storage boundary as an L0 measure term [MSR-2].
For an array-, buffer-, or slice-index node, support includes its collection's resolved element storage and the complete support of its offset; it is not a length node, so any potentially overlapping element write kills the goal.
Literals and named const values add no support.
An evaluated-value datum adds no support: it denotes an already evaluated captured value, is queried only at its one immediate call or operation judgment, never enters an ordinary goal-origin map, join, or loop-carried source fact, and never causes the original expression to be reevaluated.
Every borrow or box/arena holder used by a goal's resolved place is also a support member.
The two signs of one goal have identical support.

A requirement or verified postcondition fact has exactly the ordinary L0 or opaque-goal support of its normalized relation after the rule's stated substitutions.
An affine invariant conclusion is different: it is a theorem over the immutable mathematical value-image atoms captured when that invariant occurrence was proved, not a proposition that rereads the mutable source bindings whose spellings formed it.
A write, consume, or scope exit changes or removes the current binding-to-image map but does not make an already proved theorem about the old image false; a live alias may therefore continue to use it, and a named `proof_use` source denotes exactly that immutable theorem while its invariant declaration remains in lexical scope [INV-1, PRF-1].
Without a current value image or another retained theorem connecting an old atom to a submitted target, an unreachable old atom cannot help prove that target.
Header assumptions are removed on every edge leaving their loop, while local invariant conclusions follow [ENT-5]'s canonical control-flow intersection independently of their proof-only names.
The compiler neither removes one constructor and reruns the body nor computes a masked fact state to decide whether any fact was necessary.

An S12 relation, a narrow-receiver relation, and a relation transported through `value_if` have exactly the ordinary L0 support of their terms after the route's stated substitutions.
The callee summary reference, call or delivery edge, pre-transfer substitution record, and a result or payload binder already replaced by its receiver are checked metadata, not additional support.
A route whose substitution leaves a non-[ENT-2] operand never creates an L0 fact.

Independently of relation flow, FN-9 entry-image stability begins live for each referenced parameter datum at function-body entry.
The same overlap, holder, consume, effect, scope-exit, and counted-continuing-kill classifications below permanently invalidate it; for a measure datum the descriptor-storage boundary is the same as for ordinary measure support [MSR-2].
A structural merge retains stability only when every reaching input retains it, and a loop head removes stability for every datum a continuing kill may invalidate.
Neither contradiction, re-establishment of a fact, assignment of an equal value, nor a later iteration restores stability.
This metadata creates no snapshot, term, relation, signed goal, or runtime action.

An L0 fact or opaque signed goal dies at the earliest of: (a) a [SET-1] `set` or [SET-2] `replace` commit whose resolved target [SET-1, SET-2, OWN-5] overlaps, under [OWN-7]'s overlap relation, the resolved place of any support member, or the compiler-owned update of a `for_stmt` binder when that binder is a support member — a measure term's support is its place's descriptor storage [MSR-2], so a write to that place or to any prefix of it kills that place's measures, a write at an element position of it kills the measures of the written element and none of its own, and a write to a sibling field kills neither; a [SET-1] `set` commit whose right-hand side has one fragment type evaluates that right-hand side to its commit value before this kill, and after the kill exactly [ENT-3.S5]'s applicable post-write image is established; (b) a call — ordinary function or table operation — one of whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller place or origin set containing a place that overlaps [OWN-7] the resolved place of any support member; the projection is exactly [EFF-2]'s, so a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place, and a call whose row carries no `writes` kills nothing; how far such a projected write *reaches* — the actual's descriptor storage, or only a viewed range's element storage — is classified by [CALL-1] through [CALL-3] from the callee's declaration and by nothing else [CALL-5], and neither the actual's spelling nor the callee's body is consulted; (c) a consuming use [OWN-1] of any support member's root; (d) an edge leaving the region of any borrow holder in its support, leaving the lexical scope of any support binding, or leaving the owning counted construct of any capture term in its support, region exit [OWN-3] included.
Immediately before every specification-ordered batch of kills (a)–(d), materialize the complete [ENT-4] least closure while every pre-kill term and goal remains available; then remove exactly the conclusions whose own support the batch kills. This includes consequences obtained through transitive bounds, implicit type or constant bounds, disequality strengthening, and opaque-goal closure; a partial projection over explicit bound edges is not equivalent. A post-write image or other post-event establishment occurs only after that event's kill as its source rule states.
Scope exits are edge events. After every earlier event and its stated post-event image on that edge, materialize the complete reaching closure, apply scope kills (c) and (d), and close the surviving state before any query or join at the target.
A materialized conclusion survives exactly when its own support survives. Thus an arm-local term may be an intermediate vertex proving a relation among outer values, but no fact or goal whose conclusion still names that local, its holder, or its storage survives the scope into a join.

An ordinary user-call boundary has one order in the current ProofContext.
First, at the pre-transfer point, complete the A0 judgments, retain each referenced formal's exact pre-transfer substitution, and judge the actual obligations and FN-8 goal.
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
A `loop_stmt` with no `break` resolved to it has an empty join and therefore the contradictory state, consistent with that continuation being unreachable in truth while the conservative graph keeps it reachable.
A `propagate` right-hand side's `Err` edge leaves the function; its normal continuation keeps the preceding state subject to the initializer call's own kill events (b) and (c), and its binder gains no fact.
For every join above, contributing arm, branch, `give`, and `break` edges use their source `NodePath` order.

The continuation of a `for_stmt` is the join of its structural false-header edge and every `break` edge resolved to that counted loop, each taken after the applicable pre-exit closure, all binder, capture, and body-scope exit kills, and surviving-state closure.
The false edge always exists in the conservative graph [FN-1], so this join is never empty.
A counted continuation orders that false-header edge first and its contributing `break` edges in source `NodePath` order after it.
A `break` resolved to an enclosing loop, a `return`, or a `propagate` error edge contributes nothing there.
Because the counted binder and both captures are out of scope before the join, no S11 body fact, capture fact, or raw `binder = upper_capture` fact reaches the continuation.
An [INV-1] exact-exhaustion conclusion reaches the continuation only when the identical outer-value conclusion is present on every reaching input of this join; in particular, a `break` edge receives no conclusion from the false header and therefore removes a conclusion not independently true on that edge.

For an ordinary loop L, the conservative head state is the state before L minus every fact having a support member that a continuing kill event of L may kill.
A kill event (a)–(d) placed inside L's body, at any nesting depth, is continuing for L exactly when some path of the conservative structural normal-control graph [FN-1] leads from the edge carrying that event to L's body entry without leaving L's body — that is, exactly when an execution taking that edge can reach a later iteration head of the same loop.
Every other kill event inside the body is not continuing and is not scanned: an event on or reachable only through a `break` edge resolved to L or any enclosing loop, a `return` edge, or a `propagate` error edge leaves L for the loop's continuation or the function-return sink [FN-1, ERR-3], and no iteration head of L is reached from it without first re-entering L from outside, where the enclosing flow supplies the state.
A kill inside a nested ordinary or counted loop whose continuation lies inside L's body is continuing for L, including the kills carried on that nested loop's own `break` edges, because L's body entry is reached from that nested loop's continuation without leaving L.
Without a parenthesized invariant header, exactly those surviving facts hold at every iteration head; establishment and kills then proceed ordinarily within the iteration, and no fact established inside an iteration survives to the next iteration's head.
With a header, [INV-1] first proves every header invariant simultaneously in the complete state before L without assuming any invariant from that header.
After that base batch succeeds, the complete header batch is added to the conservative head state as the assumptions for an arbitrary iteration.
At every reachable normal body fallthrough, after ordinary statement effects, closures, and body-scope cleanup, [INV-1] proves the complete header batch again over the current value images while assuming the current-iteration header batch.
Only the proved header batch, not an arbitrary body-established fact, is reintroduced at the next ordinary-loop head.
If no normal fallthrough reaches the backedge, the preservation batch is vacuous.
A fact a non-continuing edge kills is still removed on that edge: the continuation join above takes each `break` edge after that edge's scope-exit kills, and an edge to the function-return sink reaches no queried program point, so narrowing this scan opens no path on which a dead fact is read.

A counted `for_stmt` uses one compiler-owned structural binder recurrence.
An [INV-1] header invariant changes no runtime edge or recurrence: the writer supplies the proposition, while the checker proves its base and arbitrary-backedge obligations against this fixed recurrence rather than inventing an induction hypothesis.
First its preheader establishes the S11 capture equalities and binder initialization and closes that complete post-capture state under [ENT-4].
Second, [INV-1] proves the complete header batch simultaneously in that closed post-capture preheader state, without assuming any member of the batch.
An event in the body, including the hidden normal-fallthrough binder update and body-scope cleanup, is continuing exactly when some path of the conservative structural normal-control graph [FN-1] leads from its edge through the counted header to a later entry of that same body without leaving the counted body; an event on or reachable only through a `break` resolved to that counted loop or an enclosing loop, a `return`, or a `propagate` error edge is not continuing.
Kills inside a nested ordinary or counted loop are classified by that same positive reachability predicate.
Third, its conservative head state is the closed post-capture state minus every fact having a support member that a continuing kill event may kill, and the complete proved header batch is then activated there.
On each true header edge, S11 adds the two structural body-entry bounds to that state.
The hidden binder update kills every fact supported by the binder before a later header, while S11 re-establishes only its two stated bounds after the next true guard.
At every reachable normal body fallthrough, [INV-1] proves the complete next-header batch after ordinary body effects and cleanup and after substituting the compiler-owned `binder + 1` image for the binder; the complete current header batch is available as an assumption, and no target may assume its own next-header conclusion.
This order is fixed: preheader establishment and closure, simultaneous base proof, continuing-kill subtraction, header-batch activation, S11 body-entry establishment, body flow including body-scope cleanup, formation of the compiler-owned `binder + 1` image and proof of the hidden update's representability, then simultaneous next-header proof over that image.
Neither endpoint is evaluated again and neither capture-to-endpoint equality is re-established after the preheader.
Therefore a continuing write to a mutable endpoint source kills the direct capture-to-source equality, while a consequence already closed in the preheader whose support contains only immutable captures and other still-live terms may soundly survive.
No other fact established inside one counted iteration survives to a later counted head; a body `invariant_stmt` may nevertheless serve as an ordinary proved premise for the next-header batch at the backedge where it is live.

[ENT-6] Every proof-required partial operation and callable contract creates one typed Goal at the source node that would otherwise perform it.
A Goal is normalized by its owning rule from that node's exact operands, types, layout, target, ownership state, and effects.
The checker submits the Goal with the current [ENT-3] ProofContext to one fixed deterministic domain checker; a consumer never selects a checker, retries another route, or treats a diagnostic derivation as a second acceptance pass.
A Goal that no fixed domain discharges is rejected by its owning source rule before lowering.
No runtime guard, trap, fallback operation, optimizer assumption, timeout result, or writer-stated conclusion can replace discharge.

The affine part of one ProofContext contains a current-value image for every live own-mode integer binding that this rule can represent.
An image is one exact mathematical form `c + Σ ai*xi`, where c and every ai are checked i128 integers and each xi is one compiler-owned immutable value atom carrying its exact source integer type interval.
This map is not a second source fact database: it records what value a binding currently denotes so that every consumer normalizes its one submitted proposition over the same atoms.

Image formation is exactly the following structural transfer.
An own integer parameter and any integer result whose listed form below is unavailable receive one fresh atom with that type's complete interval.
A typed integer literal or named integer const has its mathematical constant image; reading or ordinarily copying a live own integer binding reads its current image; and a total value-preserving integer `cvt` keeps the operand image.
After its ordinary IntegerDomain obligation has succeeded, an exact integer addition or subtraction has the sum or difference of its operand images, and an exact integer multiplication has the scaled image when either complete operand image is a mathematical constant; every other integer-producing operation receives a fresh atom.
An expression that may write or consume a place before producing its result receives a fresh result atom rather than an image reconstructed across that effect.

An ordinary `let` installs the initializer image at its new binding after the initializer's effects.
A whole-binding `set` first forms the right-hand-side image from the entering values, performs the ordinary target kill, then makes the target denote that image; a projected or indexed set does not replace the root binding's scalar image.
A whole-binding `replace` additionally makes its result binding denote the target's pre-write image.
A consume or scope exit removes the affected binding-to-image entry but does not alter an immutable theorem over the former atoms.

At a control-flow join, a binding keeps an identical image held on every non-contradictory input.
Otherwise every input image is first normalized: each delta atom an earlier join minted is folded back into the constant interval it stands for — that atom's coefficient times its interval, added to the input's constant — leaving one non-delta nonconstant form and one closed constant interval.
If every normalized input then has one identical non-delta nonconstant form, the joined image is that common form plus one fresh delta atom whose interval is exactly the minimum through maximum of the inputs' constant intervals; otherwise the binding receives one fresh full-type atom.
An input carrying no delta atom normalizes to its own nonconstant form and the closed interval of its own constant, so this is the earlier rule wherever no join has run.
A delta atom is an ordinary shared atom everywhere except a join, so a relation formed over it after one join still holds at the next; folding at the join is what makes the joined image the same whether the writer spells one branch set as nested conditionals or as one flat `match`, so acceptance never depends on the shape of the join.
The join never equates distinct atoms merely because two source expressions have the same spelling.
A loop's continuing-kill construction similarly replaces every loop-carried mutable binding by a fresh header atom; proved header invariants are the only source-written relations reintroduced over those header images.
The counted binder uses the captured lower image for its base, one fresh header image for an arbitrary iteration, and the exact `header_image + 1` form for a reachable next-header obligation.

All of these transfers are source-structural, checked to the same affine formation ceilings, and independent of proof success order.
They create no independently selectable premise except the invariant conclusions and specification-fixed automatic images expressly listed below.

Every live measure term carries one compiler-owned immutable affine atom as its image, over the complete `u64` interval, minted once per measure term and never retargeted [MSR-4].
That atom is not a source binding and is not a written `affine_factor` [INV-1]; it exists so that the automatic derivation below can range over measure terms, and it dies exactly when its measure term's support dies [MSR-2].
The closed L0-to-affine index is formed on demand from exactly Z, each live measure term, and each live own integer binding having both its ordinary [ENT-2] place term and a current affine value image.
For every ordered pair of those candidates whose closed L0 state has a tightest bound `left_term - right_term <= c`, substitute the candidates' current affine images to form `left_image - right_image <= c`.
For one canonical affine coefficient vector retain only the smallest upper bound; a single image whose coefficients or bound are unrepresentable in i128 is skipped and cannot suppress another image.
These retained inequalities are the `strongest canonical L0 images` below.
They are an ephemeral goal-query index over already-closed L0, not copies published into the automatic affine-premise list.

For a normalized affine inequality A, `DIRECT(A)` is exactly the following nonrecursive check, in this order: a contradictory current combined state under [ENT-4]; the strongest canonical L0 image having exactly A's canonical coefficient vector and an upper bound no greater than A's; or fixed interval substitution of every remaining atom, using its lower endpoint for a negative coefficient and upper endpoint for a positive coefficient, where each endpoint is the strongest closed L0/type bound for that atom.
`DIRECT` never selects or subtracts a published affine premise.
Every invariant conclusion and specification-fixed automatic image is appended when established to one automatic affine-premise sequence; its source category is diagnostic evidence and never partitions proof authority.
The capacity identity of [MSR-2] is one such specification-fixed automatic image: for every live measure place P the two inequalities `len_of(P) + room_of(P) - cap_of(P) <= 0` and `cap_of(P) - len_of(P) - room_of(P) <= 0` are appended over that place's three measure atoms, in that order, with the empty support every standing fact has, and they are appended when P's measure terms become live rather than by any operation's post-state.
At a join, an inequality survives exactly when the canonically identical inequality is present on every non-contradictory input under [ENT-5]'s all-predecessor rule; contradictory inputs are neutral, and if every input is contradictory the affine sequence is empty because L0 already proves every target.
The surviving sequence is ordered by the first occurrence of each canonical inequality in the first non-contradictory structural predecessor under the edge orders fixed above.
For each surviving inequality and each non-contradictory predecessor, the retained representative is that predecessor's occurrence with the fewest active-loop dependencies, ties retaining insertion order; the joined dependency set is the sorted union of those representatives' dependency sets.
This preference prevents an earlier loop-local duplicate from hiding a later loop-independent proof of the same theorem; source and derivation evidence otherwise selects diagnostic parents only.
At every query, canonically identical inequalities are represented once at their first occurrence in this sequence.
Ordinary L0 relations are not copied into that list.

`AUTO(T)` for one affine target T exhausts exactly these finite families: `DIRECT(T)`; for every listed premise P, form `S = P` and check `DIRECT(T - S)`; for every unordered listed pair P,Q including P equal to Q, form `S = P + Q` in pair order and check `DIRECT(T - S)`; and for every strongest canonical L0 image R, form `S = R` and check `DIRECT(T - S)`.
Every premise has coefficient one; forming S and then the one residual uses checked `i128` arithmetic in the stated order.
An unrepresentable candidate is skipped, not accepted and not allowed to suppress a later candidate.
Every accumulated S carrying at least one term is also offered in its integer tightening: because every atom denotes a mathematical integer, S divided by a positive integer factor k dividing each of its coefficients proves that divided left-hand side against the mathematical floor of S's bound divided by k, taken toward negative infinity, and `DIRECT(T - S/k)` is checked immediately after `DIRECT(T - S)`.
Exactly two factors are formed, in this order: the k for which S's coefficient vector is exactly k times T's, and the greatest common divisor of S's coefficient magnitudes; each is read from those two vectors in checked `i128` arithmetic, and a factor of one, a factor not dividing S exactly, and an unrepresentable division each add no candidate.
These complete zero-, one-, unordered-two-, and final L0-image families, each with its two integer tightenings, define AUTO's semantic candidate set.
Within one family, premises use the traversal above, unordered pairs use lexicographic `(first, second)` order with `first <= second`, and strongest canonical L0 images use Z followed by live own integer bindings in their compiler-owned source-allocation order for each ordered `(left, right)` pair, retaining the first occurrence of a coefficient vector unless a later image has a strictly smaller upper bound.
An unproved result exhausts the whole set; a proved result may stop at the first witness in this fixed traversal because later candidates cannot revoke it, so traversal order selects only retained diagnostic parents and cannot change acceptance.
`AUTO` does not recurse, saturate newly derived residuals, search for a multiplier outside the two integer-tightening factors fixed above, choose a subset larger than two published affine premises, or publish an intermediate result.
Consequently an author can determine from this rule alone whether a target is automatic: a derivation outside these exact shapes requires the explicit [PRF-1] `proof_use` list rather than compiler probing.

An [FN-8] Signed Goal query first applies the ordinary positive and negative [ENT-4] disposition to its complete root.
When neither sign is ordinarily derivable, its one remaining positive-proof route recursively follows exactly [ENT-4]'s fixed Boolean introduction table over the already-written goal tree: positive `band` and negative `bor` require every child in source order; negative `band` and positive `bor` visit every child in source order and retain the first successful witness; and `bnot` checks its sole child under the opposite sign.
`bxor` has no introduction route.
At each visited child, when the child root is `<=`, `<`, `>=`, or `>` over values having current affine images or measure terms, the checker normalizes that exact truth sign to one affine inequality; the child is then submitted to [MSR-4]'s disposition in the same ProofContext, which takes the ordinary [ENT-4] proof first and the normalized inequality at its affine steps.
Successful children are joined only by the stated Boolean introduction node; they publish no child, parent, L0, or affine fact, and an unsuccessful candidate changes no later candidate or acceptance result.
This structural traversal invents no proposition, connective, rewrite, premise, or coefficient and is part of the single Signed Goal query rather than an [FN-8] retry or fallback.

[MSR-4] One numeric goal disposition, shared by every consumer.
This rule states once the complete ordered derivation of a numeric goal, and every consumer submits a goal and receives that disposition:

```text
1  contradiction in the current combined state                                  [ENT-4]
2  the exact signed fact, when the goal has one                                 [ENT-4]
3  the closed L0 state                                                          [ENT-4]
4  DIRECT over the affine domain                                                [ENT-6]
5  AUTO over the affine domain, exactly the zero-, one-, unordered-pair
     and final-L0-image families with their two integer tightenings             [ENT-6]
6  the affine-left / L0-right bridge, for every live measure term, every
     measure datum, and every live own integer binding having a current image   [ENT-6]
```

Step 2 applies exactly when the submitted goal has an exact signed identity in [ENT-2]'s finite universe; a normalized affine relation with no opaque goal skips it.
Step 6 visits its candidates in compiler-owned source-allocation order, measure terms before own integer bindings; when closed L0 has the tightest bound `m - r <= c` relating a candidate m to the goal's right-hand term r, it submits the one exact residual target to `AUTO` and composes a success transitively with that L0 bridge.
An unavailable image or an unrepresentable candidate is skipped without suppressing a later candidate; a goal no step discharges is unproved and is rejected by its owning rule.

The consumers are exactly [OP-4] subscript bounds, [OP-2] integer domain, [OP-9] allocation fit, [FN-8] requirements, [FN-9] normal-result relations, and [INV-1] invariant targets.
Each keeps its own normalization — which proposition it forms from its source node — and none keeps a route grant of its own: an operation adds a goal, never a route.
The per-family route lists this rule replaces are retired, and a family paragraph below states its normalization and then submits.

This rule is not widened.
A derivation outside these exact automatic families requires the explicit [PRF-1] `proof_use` list; this rule admits no additional automatic candidates.
Step 1 is the disposition's own hazard and is stated first because it is real: in this language an inconsistent published relation is not a wrong fact, it is every fact, which is why [CALL-6] carries a consistency check at the declaration that publishes one.

The numeric relation domain attaches exactly four normalized families in this version.
For every source subscript `P[i]` — read, write, and [SET-1] target position alike — SubscriptBounds is `i < len_of(P)`, normalized `i - len_of(P) <= -1`, at that subscript's `psuffix` node.
There is one obligation per subscript in a chain.
The offset has exact type `own u64` [OP-4], so the relation is over the two u64 mathematical values, and it is a logical offset [MSR-1].
A subscript has no separate opaque signed-goal identity for its own bounds obligation; after that obligation succeeds, its selected structural index row may occur as a value child of another exact Goal as [ENT-2] fixes.
For an `array<T, N>` whose selected N is a concrete value in this instance, the normalization also offers `i <= N - 1` composed with the implicit L0 equality `N = len_of(P)`, which is a second proposition for the same obligation and not a second route.
The normalized target is then submitted to [MSR-4]'s disposition.
A refuted or unproved occurrence is an OP-4 rejection and publishes no checked program.

IntegerDomain attaches one obligation to every proof-required exact integer occurrence [OP-2] at its `infix` or `call` node.
Its canonical goal is always the corresponding total `.defined` operation with the same selected concrete type and complete ordered operand identities.
Each operand uses its stable value expression when available after all nested obligations have succeeded, otherwise that obligation's occurrence-local evaluated-value datum [ENT-2].
Its normalizations are the finite L0 normalization, the affine normalization, and the nonconstant-product rule below; each yields one proposition, and each is submitted to [MSR-4]'s disposition in that written order, the goal itself supplying step 2's exact signed identity.
One derivation root aggregates a successful route's parents in the fixed component order below; components are internal derivation nodes, not separate obligations.

The finite L0 normalization for exact add, subtract, and multiply applies when at least one operand is a constant and is the following two-bound proof over mathematical integers, upper component before lower component.
For `t + c` and `c + t`: `t - Z <= max(T) - c`, then `Z - t <= c - min(T)`.
For `t - c`: `t - Z <= max(T) + c`, then `Z - t <= -min(T) - c`.
For `c - t`: `t - Z <= c - min(T)`, then `Z - t <= max(T) - c`.
For `t * c` with c > 0: `t - Z <= floor(max(T)/c)`, then `Z - t <= -ceil(min(T)/c)`; with c = 0 both components are `Z - Z <= 0`; with c < 0: `t - Z <= floor(min(T)/c)`, then `Z - t <= -ceil(max(T)/c)`.
For two constants, normalization is ground true exactly when the mathematical result belongs to T and ground false otherwise.

When already evaluated operands have current affine images, the affine normalization forms the exact mathematical result image for add or subtract, for negate, and for multiply when either complete operand image is a constant.
It submits, in order, `result <= max(T)` and `min(T) <= result` to `AUTO`; both must succeed.
Thus two nonconstant affine add or subtract operands have this automatic route even though they have no finite L0 normalization.

When neither multiplication operand image is constant but both are affine, the only nonlinear automatic rule is the fixed interval product.
For each operand independently, start with its direct closed L0/type interval, then visit each canonical premise once in the AUTO traversal and retain a strictly tighter endpoint when subtracting that one premise followed by fixed interval substitution proves it; the selected lower and upper endpoints are each re-proved by `AUTO`.
Form the four products of the two inclusive endpoint pairs with checked `i128` arithmetic.
The multiplication domain succeeds exactly when all four are in `min(T)..=max(T)`.
The rule publishes no product inequality over the operands and no intermediate premise; the one thing it publishes is the constant interval its own four products bound, established by [ENT-3.S14] on the value the admitted multiplication binds.

The finite L0 normalization for exact division and remainder is `d != Z` together with a second component: ground true for unsigned T, and `(n != min(T)) or (d != -1)` for signed T, testing the dividend witness before the divisor witness.
It is refuted when `d = Z`, or when T is signed and both `n = min(T)` and `d = -1` are derived.
If that finite route is unknown and the operands have affine images, unsigned division or remainder submits `1 <= d` to `AUTO`.
For signed T, take in order each nonzero target `d <= -1`, `1 <= d`, crossed in that order with each overflow-safe target `min(T) + 1 <= n`, `d <= -2`, `0 <= d`; the first pair whose two members both succeed under `AUTO` proves the domain.
For exact negate and absolute, the finite L0 normalization is `x != min(T)`; the affine route uses the two result-range targets above for negate and `min(T) + 1 <= x` for absolute.
For exact shift, the finite L0 normalization is `k < K`, equivalently `k - Z <= K - 1`, where K is the selected value type's bit width; the affine route submits `k <= K - 1` to `AUTO`.
A refuted or unproved IntegerDomain Goal is an OP-2 rejection carrying its canonical `.defined` spelling.
The `.defined` Goal itself is not an invariant target: when an affine route needs writer guidance, a preceding proved invariant establishes the required operand or interval relation, optionally using [PRF-1], and the operation's fixed checker consumes that published relation.

AllocationFit attaches one canonical `buffer_fits::<T>(n)` Goal to every `buffer_new(n, v)`, and `buffer_fits::<Option<T>>(n)` to every `buffer_vacant::<T>(n)`, at that `call` node [OP-9].
Its length child uses the same stable-or-occurrence-local identity rule as IntegerDomain, so every AllocationFit occurrence has one canonical Goal.
Its normalization is `n <= floor((2^64 - 1) / stride_ceiling(S))` for the selected stored type S, and that proposition, with the Goal itself supplying step 2's exact signed identity, is submitted to [MSR-4]'s disposition; a derived false comparison refutes.
A refuted or unproved occurrence is an OP-9 rejection and creates no allocation or runtime operation.

Initialization, ownership and loans, state effects, layout and address formation, selected-target integer domains and parallel permissions keep their own finite proposition and checker domains under their owning numbered rules.
They use the same fail-closed Goal/checker principle; a later domain needing a source numeric conclusion consumes the checked conclusion retained from ProofContext rather than repeating its derivation. They are not encoded as numeric L0 relations merely to make one universal solver, and they do not become a second authority for accepting source propositions.
Each checker has a specification-fixed finite algorithm whose complete work is a deterministic function of its source-derived input, a unique closure or result, a deterministic diagnostic order, and no timeout-selected acceptance.

The mechanical repairs for an unproved Goal are a dominating source branch whose false edge handles the domain outcome, a preceding proved invariant whose optional [PRF-1] block names sufficient premises, or a verified callee relation [FN-9].
For a subscripted offset that is not itself an [ENT-2] term, first bind the inner read with one ordinary `let` and, where required, one total `cvt`; its own inner obligation is discharged independently.
Writing a proposition without one of these derivations establishes nothing.

Each concrete obligation identity is `(concrete function instance, exact source NodePath, family ordinal)`.
SubscriptBounds, IntegerDomain, and AllocationFit use ordinal zero.
A requirement occurrence is `(concrete function instance, requires_clause NodePath)` [DIAG-2].
These identities do not participate in Goal equality [FN-8].
The checked program retains the accepted Goal, its deterministic derivation root, and its erased disposition for diagnostics and proof consumers.
Internal derivation metadata is only the diagnostic explanation of that acceptance decision and establishes nothing independently.
[INV-1] A `header_invariant` and an `invariant_stmt` are two placements of the same proof-only declaration: the writer states that one ordered affine relation holds at that exact source point, and the checker must prove it before the relation gains authority.
A loop-header placement additionally creates induction obligations because control may enter that point from the preheader and from a backedge; a body placement creates only the one ordinary program-point obligation in its entering ProofContext.
The spelling `invariant` therefore describes the writer-visible meaning in both positions, while the control-flow owner determines how many incoming-edge obligations exist.

The `compare_op` of a `header_invariant`, an `invariant_stmt`, or a relation-form `use_premise` must be exactly `<=`, `<`, `>=`, or `>`; it selects a proof-domain relation over its two affine expressions and performs no [OP-1] operation, and `==` or `!=` in that position is a hard error at the `compare_op` node. It cites INV-1 in a `header_invariant` or an `invariant_stmt` target, and PRF-1 in a `use_premise`, because a relation source is owned diagnostically by PRF-1 as that rule states; the restriction itself is one rule stated once, and only its citation follows the owning position.
The checker normalizes `a <= b` to `a-b <= 0`, `a < b` to `a-b <= -1`, `a >= b` to `b-a <= 0`, and `a > b` to `b-a <= -1`.
Equality, disequality, and every other Bool root are outside this version's invariant surface.

An `affine_expr` denotes a mathematical integer expression and never a runtime evaluation.
At a counted-loop header an IDENT may resolve to that header's `for_binding` binder or to a live own-mode integer value in the preheader.
At an ordinary-loop header it may resolve only to a live own-mode integer value in the preheader.
At an `invariant_stmt` it may resolve only to a live own-mode integer value in the statement's entering lexical context.
An `affine_factor` `call` is admitted exactly when it is one of the four measure formers `len_of(P)`, `cap_of(P)`, `room_of(P)`, `head_of(P)` [MSR-1] over an admitted measure place [ENT-2] clause (b), written as [MSR-5] writes it: one `atom_list` operand naming that place, no written type argument, no `fieldinit_list`, and no second operand.
Such a factor denotes the [ENT-2] measure term over that place, of fragment type u64, lifted to its mathematical integer value like every other atom, and its support is [MSR-2]'s: an event killing that term retargets the factor's image exactly as a write to a named local retargets that local's, so no conclusion resting on it survives the write.
The place it names resolves in exactly the context this rule gives an IDENT — the preheader at either loop header, the statement's entering lexical context at an `invariant_stmt` — except that it names a live own-mode value of measured type rather than a live own-mode integer, and it is never a counted header's `for_binding`.
A subscript inside that place is an ordinary [OP-4] occurrence: its offset resolves in that same context and is one of the offsets [ENT-2] clause (b) admits, and it owes `i < len_of(base)` against the prefix reaching its base, judged where the relation is written — at the loop header in its entering ProofContext, at an `invariant_stmt` in that statement's entering one — exactly as one written at a measure former the program executes is judged at the read [MSR-4].
A measure over a place whose subscripts are not all discharged is no term here either, so the relation names a slot the run has or it names nothing.
Any other `call` in that position is a hard error citing INV-1 at the `call` node, carrying the four measure formers as what the position admits.
An `affine_factor` `atom` is admitted exactly when it is one bare `place` whose `pbase` is an IDENT and which carries no `psuffix`, or one integer literal; [GRAM-4]'s production is shared with a contract clause [MSR-5] and carries the wider factor set that clause needs.
Such a `pbase` resolves to a live own-mode integer value as this rule states above, or to an in-scope integer-typed const generic [MSR-6], whose image is the constant [ENT-2] clause (c) already fixes: a concrete instance reads its mathematical value and the one source-canonical symbolic instance reads the symbolic constant term, which no [ENT-5] event kills and whose support is empty.
A const generic is not an integer literal, so it never supplies the one direct literal operand a non-unit `*` requires.
An integer-typed named const is admitted and denotes the one closed value it declares, folded to that value at formation; it is already an [ENT-2] constant term, so it means in a relation exactly what it means everywhere else. An integer-typed const generic is admitted as the paragraph above states [MSR-6], symbolic in the source-canonical instance and its value in a concrete one, and is not the closed-value admission. Construction, dereference, subscript, field selection, allocation, borrow holders, moved values, and every other runtime expression form are not admitted as affine atoms in this version, and each is a hard error citing INV-1 at that `affine_factor`.
Every literal and local value has its exact closed source integer type and is lifted to its mathematical integer value; mixed widths and signedness create no runtime conversion.
`+` and `-` associate from left to right in source order.
`*` is admitted only when at least one direct operand is an integer literal; two local or parenthesized nonliteral operands are non-affine.
Parentheses alter grouping but never turn a composite expression into a direct literal factor.
Formation and normalization use checked `i128` arithmetic and the fixed structural ceilings of 4096 scheduled expression nodes, 4096 input terms, and 4096 normalized result terms.
Overflow or a structural-ceiling excess rejects at the owning invariant; there is no cumulative work or elapsed-time ceiling.

A `for_stmt` header is the complete parenthesized list fixed by [GRAM-4]: its first and only `for_binding` is followed by zero or more comma-separated `header_invariant` clauses, with no trailing comma.
A `loop_stmt` either has no header parentheses or has one nonempty parenthesized comma-separated list containing only `header_invariant` clauses, with no trailing comma.
A header invariant has no proof block and no `proof_use`; a complex base is stated by a preceding `invariant_stmt`, and a complex backedge is stated by an `invariant_stmt` on the reaching body path where its local premises are live.
All invariant names in one header are distinct and enter scope simultaneously only after the complete header [INV-1].
Their conclusions form one simultaneous batch.

For the base batch, the checker submits every header target to [MSR-4]'s disposition in the complete preheader state and assumes no conclusion from that same header.
If any base target fails, no header conclusion is published.
After all bases succeed, the complete batch is available as the current-iteration assumption throughout the body.
For every reachable normal backedge, the checker proves every next-header target in one batch from the complete state on that edge while the current-iteration header batch is available; a target never assumes its own unproved next-header result.
For an ordinary loop the next-header target is the same written relation over the current backedge value images.
For a counted loop each binder occurrence in the source relation is rendered and proved as the current binder's exact mathematical `+ 1` image, and every other mutable atom uses its current backedge image.
This is the induction step for an arbitrary iteration, not a check of a particular second iteration.
A backedge batch is vacuous when no normal body fallthrough reaches it.
A `break`, `return`, or `propagate` error edge creates no backedge obligation.
Failure reports the invariant name, whether the failed incoming edge is base or backedge, and the complete required source-level relation after the counted next-state substitution; an internal affine term or value-image identifier is never the writer-facing residual.

A reachable `invariant_stmt` is checked exactly once in its entering ProofContext.
Without a proof block its target must succeed under [MSR-4]'s disposition.
With a proof block it is checked by [PRF-1].
It cannot assume its own target.
On success its normalized target and immutable value images become one published affine fact after the statement; the fact may serve every later goal in the declaration's dominance region and may itself be named by a later `proof_use`.
Only that target is published: formation state, certificate premises, scaled premises, accumulator values, and residuals are never added to ProofContext.
At a control-flow join, facts are compared by canonical inequality and immutable value images rather than invariant spelling or proof-source ordinal; identical conclusions reaching every non-contradictory input survive under [ENT-5].
The invariant name keeps only its lexical scope and never changes canonical fact identity.

For a counted loop whose complete header batch succeeds, the fixed exact-exhaustion rule is available only when the captured lower endpoint is proved no greater than the captured upper endpoint without using that header batch and, when a backedge is reachable, the hidden `binder + 1` update is proved representable in u64.
Initialization, every true guard, the exact hidden update, and the false guard then establish `binder = upper_capture` on the structural false-header edge.
The checker substitutes the captured upper value for the binder in each proved header invariant exactly once and exports a result only when every remaining atom is an outer value live at the continuation.
No `break` edge receives this conclusion.
The ordinary continuation join therefore retains it only when the same canonical outer-value fact reaches every input; an ordinary loop has no corresponding exhaustion substitution.
The header invariant name itself does not escape the loop body.

Every invariant form and its certificate are erased before runtime lowering.
They evaluate no expression, read or write no storage, form or consume no loan, move no value, contribute no effect, branch, allocation, call, trap, fallback, or instruction.
A malformed relation or an unproved target cites INV-1 at the smallest specified source node; a certificate-specific failure cites PRF-1 as fixed below.

[PRF-1] A proof block is an optional local linear certificate attached only to an `invariant_stmt`.
It supplies an ordered list of `proof_use` steps that tells the checker which already-provable inequalities to add; it is not allowed on a `header_invariant` clause.

A next-state relation using one current header theorem and then a value-range discharge by `DIRECT` remains inside `AUTO` and therefore has no block:

```wf
invariant next_per_byte: sum <= 255_u32 * (i + 1_u64);
```

The first example below names three listed affine premises, beyond `AUTO`'s complete two-listed-premise family.
The second uses one explicit non-unit factor, which `AUTO` never guesses:

```wf
invariant component_sum: first + second + third <= first_limit + second_limit + third_limit {
  use (first <= first_limit);
  use (second <= second_limit);
  use (third <= third_limit);
}

invariant pair_bound: first + second <= first_limit + second_limit;
invariant scaled_bound: 3_u64 * first + 3_u64 * second <= 3_u64 * first_limit + 3_u64 * second_limit {
  use 3 times pair_bound;
}
```

An IDENT premise of `use_premise` resolves in the invariant-name domain to the exact immutable normalized target published by that dominating invariant declaration; it is not reparsed using the current value bound to each source spelling.
The checked reference stores `(concrete function instance, invariant declaration identity)` obtained from that lexical resolution, never the IDENT spelling or a later spelling lookup.
Inside a loop body a header declaration identity denotes the currently activated arbitrary-iteration header theorem, not its base value images and not the still-unproved next-header target.
A relation-form source in `proof_use` uses INV-1's exact affine formation and normalization rules, including substitution of every referenced local's current value image before canonical normalization. It is owned diagnostically by PRF-1 and must itself be proved by `AUTO`.
A named source must be in lexical scope and its published fact must be available in the entering ProofContext.
Every use, named or written, is checked against the same snapshot immediately before the owning `invariant_stmt`.
No use publishes a fact, and no earlier use can help prove a later use.

A `proof_use` cites exactly one premise, its `use_premise`, and states how many times that premise is added into the certificate sum.
A relation premise is always delimited, `use (a <= b);` and `use 3 times (a <= b);`, and a named premise never is [GRAM-4]; the delimiting parentheses are the grammar's own and are not an affine grouping.
The optional `N times` prefix is that multiplicity, and it is written either as a bare decimal or as a name.
A bare-decimal multiplicity is a proof-domain positive integer factor: its canonical spelling is one decimal with no leading zero, its value must be in `2..=i128::MAX`, and omission alone means one.
It is neither a source integer literal nor a runtime type.
Zero, an explicit one, a leading zero, an out-of-range factor, a negative or typed literal, and arithmetic overflow reject.
A named multiplicity denotes the value image its declaration holds in the same entering ProofContext every premise is checked against.
It must be a live own-mode integer value binding — a `let_stmt` local, a `param`, a `for_stmt` binder, or a match binder — or an integer `const`, and its type must be **unsigned**; a signed type, a borrow, and a non-integer type each reject. Every admitted type is copy, so a moved binding is [OWN-1]'s hard error before this rule reads it.
That restriction is what makes the scaling step sound without a further obligation: multiplying a normalized `p <= 0` by a value known nonnegative from its written type yields `m*p <= 0`, while a negative multiplier would reverse the premise.
A runtime multiplicity of zero drops its premise and is not a rejection, because no written text asserts that it is nonzero.
No two `proof_use` entries may resolve to the same normalized premise, regardless of multiplicity; their total scaling must be expressed by one multiplicity on one use.
No global or subset-minimality judgment is performed on the remaining list.

The checker forms S independently of premise admission by multiplying each normalized premise by its written multiplicity and adding the results in source order with checked `i128` arithmetic; acceptance additionally requires every source to be independently admitted.
A bare-decimal multiplicity keeps that accumulation affine.
A named multiplicity does not: scaling a normalized premise by a value introduces products of two value images, so the accumulation is a polynomial of degree at most two whose nonlinear monomials exist only while S is being formed.
Before S is used, every such monomial is folded to the one value that already equals it: the value image bound by an admitted exact multiplication [ENT-6] whose two operands are the same two values, where that multiplication's own [OP-2] domain discharged over affine operand images — by the fixed interval-product rule or by an affine clause, either of which fixes the images the judgment read. A domain discharged by the finite L0 route alone records nothing, because that route reads no affine image.
A multiplication's operand and a written multiplicity name the same value when they name the same declaration, not when their images coincide. A local's image is transparent: `let stride = width + padding;` gives `stride` the image `width + padding`, so a product over `stride` and a certificate scaling by `stride` would otherwise reach the fold as different arithmetic. Each such binding therefore contributes one opaque handle, minted at its type range on first demand and killed with the binding's image, and the product record and the multiplicity both name that handle. This is the rule [PRF-1] already applies to a named premise, which resolves to its declaration identity and is never reparsed from the current value of its spelling.
A handle exists only between the fold and the residual. Once every nonlinear monomial has folded, each handle is replaced by the image it stands for, so what reaches the residual is written in exactly the terms every other premise and the target are written in, and no certificate can be discharged by a handle's own defining equation.
When several bindings hold that product, the one the target itself names is chosen, and otherwise the least; they are equal values, so either choice is sound and this one is a canonical form rather than a search.
S is that folded accumulation, and it must be an affine inequality: a certificate that leaves any nonlinear monomial unfolded rejects and proves nothing.
Nothing else in this specification carries a nonlinear term — no fact, no published conclusion, no invariant target, and no `affine_expr` [INV-1] — so the accumulation above is the complete extent of degree two in the language.
Let S be that one accumulated inequality and T the owning invariant target.
The certificate succeeds exactly when `DIRECT(T - S)` succeeds in the same entering ProofContext, or when `DIRECT(T - S/k)` succeeds there for one of the two integer-tightening factors k that [ENT-6] fixes for S and T.
In particular, when S's coefficient vector is exactly k times T's, that tightened residual is constant and the target is admitted whenever the mathematical floor of S's bound divided by k is no greater than T's bound.
This admits a target that is a fixed L0 or interval weakening of the written sum; it does not require byte-for-byte equality and it does not call `AUTO` on the residual.
The checker never guesses a source, multiplier, ordering, subset, case split, or intermediate lemma.

After every use has parsed, resolved, and formed canonically, but before premise admission and combination, the checker applies `AUTO(T)` to the target in the same entering context.
If it succeeds, the proof block is a redundant source form and compilation rejects at the owning `invariant_stmt`; removing the complete block is the mechanical repair.
This is a specification-version judgment, not an implementation-dependent warning: [ENT-1] fixes `AUTO` exactly, and changing that accepted family requires a specification amendment.
The checker does not search whether an individual nonduplicate use could be removed.
At most 4096 `proof_use` entries are admitted by one block; this is a source structural ceiling, not a work or time budget.

Only the owning invariant target is published after a successful certificate.
The `proof_use` list and all of its intermediate arithmetic are erased with the invariant and have no runtime semantics.
An unresolved invariant name is the ordinary INV-1 lexical-scope failure and forms no certificate source.
A resolved but unavailable named source, undischarged or malformed relation source, invalid multiplicity, duplicate source, arithmetic or structural overflow, unfolded nonlinear monomial in S, failed final `DIRECT` residual, or redundant block cites PRF-1 at the smallest owning source node and publishes no target.

## 16. Worked example (normative bytes)

[EX-1] The following complete program is byte-exact canonical form:

```
enum Sign {
  Neg();
  Zero();
  Pos();
}

fn sign_of(x: own i32) -> result: own Sign pure {
  doc "Conditional value produced by returning from branches (canonical for return position).";
  if x < 0_i32 {
    return Neg();
  } else if x == 0_i32 {
    return Zero();
  } else {
    return Pos();
  }
}

fn main() -> status: own ExitStatus pure {
  doc "let-initializer match with give: a conditional value bound, then reused.";
  let a = 40_i32;
  region {
    let p = &a;
    let v = match deref(p) +checked 2_i32 {
      Ok(value: w) => {
        give w;
      }
      Err(error: e) => {
        let failed = exit_status(code: 1_u8);
        return move failed;
      }
    }
    let expected = v == 42_i32;
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

## 17. Spec meta-rules (CI-checked)

[META-1] Spec-CI enforces the regularity invariants defined elsewhere: one spelling per construct [FORM-1] and a 1:1 production-to-core-tree-node mapping [GRAM-1].
Its unique machine-checked content is that no rule ID is defined twice and every cross-reference resolves [META-4].
[META-2] No context-dependent spellings or rule variants: no rule's meaning depends on surrounding context; defaulting rules do not exist.
[META-3] No rule carries an exception clause; conditional structure is expressed as total positive rules or table data.
[META-4] Every normative fact is stated once; other mentions are rule-ID cross-references.
[META-5] Every change to this artifact declares its spec delta (rules ±, tokens ±, spellings ±, exceptions ±) and its SELECTION GROUND (evidence-selected vs minimality-selected) in the change that makes it.
This document states the language and carries no commentary about its own versions: no delta declaration, no description of what a version changed, and no selection ground appear in these bytes, and a version's own such text is not retained here after it activates.
`CLAUDE.md` defines the repository's four branch-and-main rules: work-branch changes need no approval, while merging into `main` requires owner approval of the exact tested revision and the records those rules require.
DEFERRED markers are tracked specification-delta obligations and do not create another approval point.
