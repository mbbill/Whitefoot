# Kernel Specification v0.47

Status: ACTIVE v0.47
Prior versions: the immutable `spec/kernel-spec-vN.md` archives and the `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`.

META-5 delta declaration: numbered rules +0/-0 (136 remain); grammar productions +0/-0 (84 remain); changed productions 0; unique fixed lowercase grammar atoms +0/-0 (54 remain); compound punctuation tokens +0/-0 (8 remain); token bytes +0/-0; writer operation spellings +0/-0; opaque system nominal spellings +0/-0; runtime-trap families +0/-0 (0 remain); entry forms +0/-0 (1 remains); contract block forms +0/-0; system operations and declaration records +0/-0 (203 remain); exception clauses +0/-0. [ENT-3] gains no source. [INV-1] is amended; no rule id is retired.
This version admits an integer-typed named const as an affine atom. It is already an [ENT-2] constant term — clause (c) names the mathematical value of an integer literal or of an integer-typed named const in the same breath — so the exclusion made one declared value mean a number everywhere except in the relations written about it, and a limit declared once had to have its digits rewritten inline in every invariant and every `use` that named it. The const folds to its value at formation, so nothing downstream sees a new atom kind and no derivation, image, kill, or join changes; a relation naming a const renders exactly as the same relation naming its literal. A const-generic parameter is symbolic rather than closed and is not this admission: an affine factor is a number, and a symbolic constant would need an atom of its own.
Selection ground: evidence-selected, recorded in `research/investigations/binary-arithmetic/`. The accumulator sweep measured the cost as a writer-form defect rather than a proof gap: a named const in a header relation was an [INV-1] unresolved use, and binding it to a local turned the relation non-affine again, so the only surviving spelling duplicated the digits at every site. The check is that the two spellings now agree: the same loop written over `cap` and over `255_u64` produces the same required relation, byte for byte, including its rendering in a failure. Prior selection ground for v0.46's clause relation and measure atom, for v0.45's product interval, for v0.44's fact machinery, for v0.43's loop-body region and [ENT-6] join repair, for the v0.41 comparison spellings, for the v0.40 proof surface, and for [PAR-3] remains as those versions recorded it.
This version makes one relation writable in the position that needs it and gives the fact base the atom that lets a caller discharge it. Three sentences move together because none of them is useful alone. First, [FN-8] admits exact addition, subtraction, and multiplication in a clause and reads them over the mathematical integers, which is the carve-out [INV-1] already gives an `affine_expr`; a clause is erased before lowering and evaluates nothing, so a row total over the mathematical integers states a relation where it would otherwise request an operation. Exact division, remainder, negation, absolute value, and the shifts stay inadmissible, each having an input its own relation cannot state its way out of. Second, a measure term becomes an affine atom, which is the admission v0.44 recorded as deferred and did not take: one atom per measured place, minted at its full u64 range, identified by that place's root binding, with the L0-to-affine index ranging over measure terms so what is known about an object tightens it. The atom is stable while the object is, because a measure is fixed at creation and an element write never moves it [ENT-5]; only a write to the root removes it, and a join keeps it where every input agrees. Third, [ENT-6]'s affine route discharges a comparison goal whose normalization is affine whether or not that goal also projects to L0. The route is the goal's own comparison normalized, so proving it proves the goal; the projection is what the retained evidence names, not what the route requires, and a goal carrying a coefficient has no two-term projection to name.
Selection ground: evidence-selected, recorded in `research/investigations/binary-arithmetic/`. The measured case is the precondition of every expansion codec, `requires len(out) >= 2 * len(src)`, which v0.45 refuses at formation. Admitting the exact rows alone makes it form and leaves it unprovable at every caller — measured, at a call site where the goal reads `8 >= 4 * 2` — because L0 is a two-term difference bound that carries no coefficient and the affine layer had no measure atom. Adding the atom alone leaves the route unreachable, because the affine target is built and never consulted for a goal with no L0 projection. With all three the case compiles and discharges, and the accepted set moves by four programs across 186 constructed probes, each one a contract the previous version could not state. The snapshot corpus holds at 491 pass and zero flips. Prior selection ground for v0.45's product interval, for v0.44's fact machinery, for v0.43's loop-body region and [ENT-6] join repair, for the v0.41 comparison spellings, for the v0.40 proof surface, and for [PAR-3] remains as those versions recorded it.
This version stops discarding a measurement the checker already performed. [ENT-6]'s fixed interval-product rule proves an inclusive interval for each operand of a non-constant multiplication and forms the four products of their endpoint pairs; the multiplication is admitted exactly when all four lie in the result type. Those same four products bound the value the operation produced, and v0.44 published none of it, so a multiplication was admitted and the value it bound carried no bound at all. [ENT-3.S14] now establishes the least and greatest of those four products on that value. Nothing about the derivation widens: the published relations are constant bounds against the zero term Z, so they are ordinary L0 facts with the bound value as their whole support, and no relation over the operands, no new term, and no automatic route by which a product enters a premise family is added. A written `use` remains the only way a product participates in a certificate, and a domain discharged by the finite L0 route or by an affine clause still publishes nothing. The measurement is stated from the one computation the domain decision consumed, so the admitted range and the published bound cannot disagree.
Selection ground: evidence-selected. Four independent sweeps against the v0.44 compiler measured what the discarded interval costs, and all four converged on this sentence rather than on the strength of the prover. The decisive pair is `E1_bounded_product.wf`, which is accepted because the interval rule discharges `r * w`, beside `E5_product_then_add.wf`, which is the same program plus `base + c` and is refused [OP-2] because the admitted product bounded nothing; supplying exactly the discarded interval as a written guard turns E5 into an accept, so that interval is the whole missing premise. The cost of supplying it by hand is one comparison, one branch and one unreachable arm per site, and a flattened index accessor pays it while widening its result to an option, which propagates a match to every caller. Over 55 constructed multi-dimensional and strided programs the amendment moves exactly two verdicts, both from refusal to acceptance, and it relocates the refusal of the guarded flattened-index programs from the arithmetic domain [OP-2] to the subscript bound [OP-4] they were always about. The programs, their compiled verdicts, and the corpus count that motivated the sweep are recorded in `research/investigations/binary-arithmetic/`. Prior selection ground for v0.44's fact machinery, for v0.43's loop-body region and [ENT-6] join repair, for the v0.41 comparison spellings, for the v0.40 proof surface, and for [PAR-3] remains as those versions recorded it.
This candidate lands the fact machinery: what a contract may be written over, and where a declared relation is computed and established.
The first amendment, [MSR-5], widens the contract-clause operand set. A `requires` and an `ensures` take a `clause_expr` whose operands are an `atom`, a `call`, or a `construct`, so a measure of a place is an operand on either side of the comparison and `ensures len(rest) >= len(out);` is a written clause where it was a [GRAM-5] parse rejection. The judgment over the widened set is the [OP-5] condition [FN-8] and [FN-9] already apply; no route, fact source, or proof authority is added. The measure formers are table data with one row in this version, `len(P)`, admitted exactly where [ENT-2] clause (b) admits a length term.
The second, [MSR-3], states one denotation per operand position, keyed on the parameter's mode, in one table rather than distributed over three rules. An `own` operand read at a caller denotes that call's **call datum** — a compiler-owned immutable term with empty support — which is what makes a relation naming a consumed operand's measure mean what it reads as at the caller. A `&uniq` parameter's measure is inadmissible in a source-declared `ensures` and is a hard error citing MSR-3, because that parameter is the one position from which a callee could leave a caller holding a measure of a value the callee replaced.
The third, [CALL-4], states the contract vocabulary over the result: a `fn_decl` declares exactly one result, so a route names no ordinal and no ordinal binder is written, and the destinations stay [ENT-3.S12]'s closed list.
The fourth, [CALL-6], states publication once — the substitution, the point at which a relation is instantiated, the point at which it is established, the destination, and the support — and adds [ENT-3]'s source S13, which mints each `own` operand's call datum at the call's pre-transfer point. A routed relation is instantiated at the call and restricted to its arm rather than deferred to it, so an event between the call and the arm kills what it removes. And a `contract_block` whose published relations are contradictory at their establishment point is a hard error at the `fn_decl`: at a contradictory point every relation and both signs of every goal are derivable, so an inconsistent contract is not one wrong fact at a caller but every fact at every caller.
Selection ground: evidence-selected. The operand widening is selected by probe `q7` of the containers-and-resources design session, which is a [GRAM-5] parse rejection of `ensures len(kept) >= 1_u64;` on the build of record, and by the same file's measurement that the surviving spelling costs one `contract_define` per measure named. The call datum and the two establishment points are selected by that design's seventh falsifier round, which reached memory three times through a published relation computed at one program point and used at another, and once through a relation set that was contradictory and therefore discharged every goal a caller submitted; the two BREAK programs and their refutations are recorded in `research/investigations/containers-and-resources/DESIGN.md` §3.K.1, §3.K.6 and §6.11. Prior selection ground for v0.43's loop-body region and [ENT-6] join repair, for the v0.41 comparison spellings, for the v0.40 proof surface, and for [PAR-3] remains as those versions recorded it.
Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification; their derivation status and open evidence are recorded in `spec/derivation/derivation-ledger.md` and relevant live `mcts_mem/` decisions): ordinary loop form (GRAM-4/6; the counted `for_stmt` is evidence-selected in v0.25 and is not this register item), statement-only match (GRAM-7), boundary annotation surface (TYPE-5), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), forced region elision (FORM-8), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), and deref prefix places (GRAM-5).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14).
The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is checker-accepted iff it parses under the canonical grammar and satisfies every machine judgment in this document.
Every proof-required partial operation is statically discharged by the deterministic checker before lowering; a writer may expose a missing fact with executed control flow, provide a machine-proved loop-header or local invariant [INV-1], direct a larger local linear derivation with [PRF-1], or publish it across a function boundary through verified contracts [FN-8, FN-9].
Failure to discharge any such obligation rejects compilation; no operation receives an implicit runtime fallback and no writer statement can request one.
There is no writer-emittable unchecked state and nothing writer-stated is trusted without machine derivation.
Runtime-origin values — parameters, system results, loaded storage, and values derived from them — enter the proof context only as typed symbolic terms; their origin neither grants nor removes proof authority.
A proposition enters the context only from a selected ordinary control-flow edge, a declaration or type fact fixed by this specification, a verified callee postcondition, or the target of a machine-proved header or local invariant whose optional [PRF-1] certificate the checker has independently discharged.
No written conclusion, origin annotation, trusted-value mark, runtime observation outside executed source control, compiler-generated record, or optimizer result is a fact source.
The gated toolchain family may implement specification-fixed primitives, but it cannot inject a writer-program fact or bypass an obligation [GATE-1, LEDGER-1].

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code.
This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.
This version temporarily leaves only external resource availability outside the source outcome model: heap exhaustion, stack exhaustion, operating-system quotas, and runtime-start resources may stop execution at the host boundary without a Whitefoot value, status, or cleanup guarantee.
That temporary scope cut does not defer static layout, stride, allocation-ceiling, address, target qualification, target-domain, parallel-independence, or bounded queue/completion proof; every one of those obligations still succeeds before the governed operation is emitted.
External resource failure establishes no source fact, grants no fallback path, and cannot turn an unproved operation into an accepted one.

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
Examples include `Result<i32, Overflow>`, `f(x: a, y: b)`, `cvt::<u8, u32>(w)`, `a <= b`, `conform i32: Zeroed`, `['r, 's]`, and `[10_u8, 20_u8]`.
The same rules render an elided region [FORM-8] with no further sentence: a borrow mode is `&u8` or `&uniq Foo`, a borrow expression is `&p` or `&uniq p`, a region-free view type is `slice<u8>` or `arena<T>`, an unnamed region block opens `region {`, and a call whose region arguments are all determined writes no `::` application at all.

Every nonempty physical line begins with exactly two ASCII spaces for each enclosing brace block.
A closing brace is rendered after reducing the depth for the block it closes.
A match-arm header is therefore one level inside its match, and statements in the arm body are two levels inside it.

The line-bearing simple productions are `field`, `variant`, `fn_sig`, `law`, `fn_bind`, `const_decl`, `doc`, `contract_define`, `requires_clause`, `ensures_clause`, `set_stmt`, `expr_stmt`, `return_stmt`, `proof_use`, `break_stmt`, and `give_stmt`, plus a `let_stmt` whose selected right-hand side is `ordinary_let_rhs`, `propagate_let_rhs`, or `replace_let_rhs`.
Each renders completely on one line, including its final semicolon.

The generically block-bearing productions are `struct_decl`, `enum_decl`, `contract_decl`, `conform_decl`, the body of `fn_decl`, `contract_block`, `region_stmt`, `match_stmt`, `value_match`, `if_stmt`, `value_if`, and `arm`.
Their introducer through `{` is one line; their children render on following lines at depth plus one; and `}` renders on its own line at the original depth.
Empty blocks still use an opening line followed by a closing-brace line.
An `invariant_stmt` ending in `;` renders completely on one line.
An `invariant_stmt` carrying a proof block renders its introducer through `{` on one line, each `proof_use` on a following line at depth plus one, and `}` on its own line at the original depth.

A `for_stmt` renders `for`, its optional label, exactly one space, and `(`; this stated space overrides the generic right attachment of `(`.
A `proof_use` whose `use_premise` is a delimited relation renders exactly one space before that premise's `(`, `use (a <= b);` and `use 3 times (a <= b);`; this stated space likewise overrides the generic right attachment of `(`, exactly as the `for_stmt` space above does, while the relation's own affine parentheses keep the generic attachment.
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
There are no boolean literals: `Bool` is a prelude enum (§15).
Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric contract (`Int` or `Float`, §15), denoting T's additive and multiplicative identity; a concrete type uses `0_i32` and the like, so there is no dual spelling.
NaN and the infinities are not literals; they are the nullary ops `fnan` and `finf` [OP-1].

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual.
The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling is the R3 one-spelling choice for the type's sole inhabitant.

[FORM-7] Numeric-literal well-formedness (R4 check-reject).
An integer literal `-?d_T` is legal where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` is legal for signed T, and `-0` is written `0`.
A float literal is legal only when it has the unique canonical spelling selected by [FORM-5] and denotes a finite value of its stated TYPE.
An out-of-range integer, a leading-zero integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value.

[FORM-8] Canonical region spelling.
A REGIONID is written exactly at the positions where this document does not otherwise fix the region denoted, and is absent at every other position, so each region position has exactly one legal spelling [FORM-1].
An absent REGIONID is neither a default nor a second meaning for a written one: the region is derived, in the class of the derived `let` binder mode [TYPE-5] and the derived match-binder mode [OWN-13], and an optional name whose absence resolves to the innermost enclosing construct is the unlabeled `break` form [TYPE-6] already carries [META-2].
Every clause below is decided by reading the owning declaration's own text, so a writer chooses the one legal spelling from the declaration alone and never from a checker verdict.
Being unnamed removes no obligation: an unnamed region has the ordinary extent, liveness, outlives, exclusivity, storage-duration, confinement, and loop judgments of the construct that introduces it [OWN-3, OWN-4, OWN-5, OWN-10, OWN-11, STOR-4].

The region positions of one `fn_decl` or `fn_sig` are its input positions — every REGIONID slot of its `param` list, in a `param`'s `mode` and at any depth of its `type` — and its output positions — every REGIONID slot of its `result_binding` `rtype` at any depth, and the REGIONID of each `arena` entry of an `allocates` effect [EFF-1].
`region_params` is a list of written names rather than a position, and a `reads` or `writes` `effect_path` names a place rather than a region [EFF-1], so neither is a position.
A region name is written at a position exactly when the same region is meant at two or more positions of that same declaration, or when the position is an output position and that region is meant at no input position of that declaration.
The first case is the only way to relate two positions; the second is the only region a caller must choose, because no actual argument determines it.
Every other position is unnamed and denotes a region distinct from the region of every other position of that declaration.
Every output position therefore writes its region: either an input position names the same region, which is the first case, or none does, which is the second.
An unnamed output position is a hard error citing FORM-8 at its `mode` or `type` production, because nothing in the declaration or at a call determines the region such a result carries.
`region_params` is written exactly when at least one name is written by that judgment; it then lists exactly those names, once each, in the order of their first written occurrence in that declaration, and it is absent otherwise.
A declaration whose written names, name multiplicity, `region_params` membership, `region_params` order, or `region_params` presence differs from that rendering is a hard error citing FORM-8, using `SourceNode` at the owning `mode`, `type`, or `effect` production of the offending REGIONID, or at the complete `region_params` when the list itself is the defect.

A `borrow_expr` [GRAM-5] writes its REGIONID exactly when the region it denotes is not the region of the innermost region block lexically enclosing it; a `borrow_expr` no region block encloses therefore always writes it.
The enclosing region blocks are the `region_stmt`s enclosing it and the loop bodies enclosing it, because every `loop_stmt` and `for_stmt` body is itself a region block [OWN-11]; a borrow written directly in a loop body therefore takes that body's own per-iteration region and is written bare.
A `region_stmt` writes its REGIONID exactly when that name occurs at least once inside its body after this rule has been applied throughout that body, and is written `region { ... }` otherwise.
A loop body introduces its region with no REGIONID at all and no position can name it, so nothing outside that body reaches it.
An unnamed `region_stmt` introduces its region exactly as a named one does [OWN-3].
A written region the innermost enclosing region block already fixes, and an absent region at a `borrow_expr` no region block encloses, are each a hard error citing FORM-8 at that `borrow_expr`; an unreferenced written `region_stmt` name is a hard error citing FORM-8 at that `region_stmt`.

A `region_stmt` that is a loop body's only statement is a hard error citing FORM-8 at that `region_stmt`, whether or not it writes a name: its block is exactly that body, the body already introduces one region over that same block [OWN-11], and the two are therefore one region under two spellings [FORM-1].
A `region_stmt` the loop body writes any other statement beside is not that second spelling and stays legal, because its block is a strict part of the body and the two extents are distinguished — [OWN-6] admits a statement-scoped child reborrow under a region whose block does not extend beyond the enclosing statement, which a one-statement block inside a longer body satisfies and the body's own region does not.
A writer therefore decides it by reading the loop body alone, asking only whether the body writes anything beside the block.
The one exception is a `region_stmt` some position inside its body must write its REGIONID at, which in a body is exactly a `targ` region argument [GRAM-5]: no implicit region has a name that position could carry, so such a block is the only spelling of its region and is admitted.
The mechanical repair is otherwise to delete the block, keep its statements as the loop body, and elide every REGIONID that named it.

A `call` whose callee resolves to a user `fn` [FN-2] or to an admitted system operation [SYS-2] writes, as the leading region members of its `::` type application [GRAM-5], exactly those of the callee's region parameters that occupy no input position of the callee's declaration, in `region_params` order.
A call whose complete type application would then be empty writes no `::` at all.
Every other region parameter is determined by the call's own actual arguments and is not written: it is that one of the actual regions at the formal positions naming it which every actual region at those positions outlives-or-equals [OWN-3].
When no actual region at those positions has that property the call is a hard error citing OWN-4, exactly as an unsatisfiable written region argument is today; the substituted region is otherwise the largest one every actual loan admits, so a related result region reaches as far as its inputs allow.
Writing a determined region parameter, or omitting an undetermined one, is a hard error citing FORM-8 at the complete `call`.
A retained-argument table operation still writes the region its row fixes, because no operand supplies it [TYPE-5, OP-1].

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
The quoted `"[0-9]+"` occurrences in the `const` production and the optional multiplier position of `proof_use` share the grammar's sole pattern predicate: each denotes one numeric-form token whose complete bytes match `[0-9]+`, and neither is a fixed atom.
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
`infix_tail` maps to the `infix` node kind: a selected tail forms one `infix` node spanning the complete `expr` — the atom and the tail — so the 1:1 production-to-node mapping is preserved by the factored recognition; its operator child is one `infix_op` or one `compare_op` node.

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
fn_decl      := program_kind? "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" result_binding effects contract_block? "{" doc? stmt* "}"
program_kind := "command"
result_binding:= IDENT ":" rtype
contract_block:= "contract" "{" contract_define* requires_clause* ensures_clause* "}"
contract_define:= "define" IDENT "=" expr ";"
requires_clause:= "requires" clause_expr ";"
ensures_clause:= "ensures" ("when" result_route ":")? clause_expr ";"
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
        | "slice" "<" (REGIONID ",")? type ">" | "box" "<" type ">"
        | "arena" "<" (REGIONID ",")? type ">" | "buffer" "<" type ">"
rtype  := mode type
mode   := "own" | "&" REGIONID? | "&uniq" REGIONID?
targs  := "<" targ ("," targ)* ">"
targ   := type | REGIONID | const
```

[GRAM-4] Statements:

```wf-ebnf GRAM-4
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | for_stmt | invariant_stmt | break_stmt | region_stmt
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
affine_factor := literal | IDENT | "(" affine_expr ")"
affine_add_op := "+" | "-"
break_stmt  := "break" LABEL? ";"
region_stmt := "region" REGIONID? "{" stmt* "}"
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
infix_tail     := (infix_op | compare_op) atom
infix_op       := "+" | "+wrap" | "+defined" | "+checked" | "+sat"
                | "-" | "-wrap" | "-defined" | "-checked" | "-sat"
                | "*" | "*wrap" | "*defined" | "*checked" | "*sat"
                | "/" | "/defined" | "/checked"
                | "%" | "%defined" | "%checked"
compare_op     := "==" | "!=" | "<" | "<=" | ">" | ">="
atom           := literal | "move" place | place | borrow_expr
call           := callee ("::" targs)? "(" ( atom_list | fieldinit_list )? ")"
callee         := IDENT | OPNAME
construct      := TYPEID targs? "(" fieldinit_list? ")"
fieldinit_list := fieldinit ("," fieldinit)*
fieldinit      := IDENT ":" atom
borrow_expr    := "&" REGIONID? place | "&uniq" REGIONID? place
atom_list      := atom ("," atom)*
clause_expr    := (atom | call | construct)
                  ((infix_op | compare_op) (atom | call | construct))?
place          := pbase psuffix*
pbase          := IDENT | "deref" "(" place ")"
psuffix        := "." IDENT | "[" atom "]"
```

[GRAM-6] There is no general operator syntax and no precedence: an `infix` expression is exactly one operation over two atoms [GRAM-5, GRAM-9], composition is by `let`, and no precedence, associativity, or parenthesization surface exists.
The `compare_op` alternatives are the six integer comparisons of [OP-1] and form `infix` expressions exactly as the `infix_op` arithmetic does; a `call` writes its type and region arguments after the `::` delimiter, `cvt::<u8, u32>(w)`, so that `IDENT "<"` begins a comparison and never a type-argument list, while a `construct` and a `type` write theirs bare.
There is no `while`.
Conditional control is type-driven with one form per class: a Bool condition takes `if`/`else`, an enum scrutinee takes `match`, and each is the sole legal form for its class — a `match` whose scrutinee has type `Bool` is a hard error citing GRAM-6 at the scrutinee `expr` node (spell `if`).
An `if` condition must have exact value mode and type `own Bool` under exactly the [OP-5] condition judgment, TYPE-7 exclusivity included; every other condition failure cites GRAM-6 at the condition `expr` node.
An `if_stmt` `else` whose block is empty is a hard error citing GRAM-6 at that `if_stmt` node (spell the else-free `if`; a `value_if`'s undelivering else is [GIVE-1]'s rejection, not this one).
An `else` whose block contains exactly one `if_stmt` and nothing else is a hard error citing GRAM-6 at that nested `if_stmt` node (spell `else if`); in a `value_if` whose else block is exactly one else-free `if_stmt`, the branch cannot deliver, [GIVE-1] owns the rejection, and GRAM-6 forms no candidate there, so the flattening fix is never demanded where the chain form could not be spelled.
A conditional value is a `let`-initializer `match` or `if` [GRAM-7, GIVE-1].
The only iteration forms are the ordinary `loop` plus `break`, and the counted ascending half-open `for` form whose complete semantics are [TYPE-5, TYPE-6, OWN-11, FN-1, ENT-2, ENT-3, ENT-5]; there is no step, reverse, iterator, or `continue` form.
The subscript suffix is a place form (its sole home); bounds semantics are [OP-4].
A `clause_expr` is the contract-clause shape and its sole home is a `requires_clause` and an `ensures_clause` [GRAM-2]: one operand, or two operands around one operator, where an operand is an `atom`, a `call`, or a `construct` and the operator is an `infix_op` or a `compare_op`.
It differs from an `expr` in exactly one way — a `call` and a `construct` may stand where an `expr` admits only an `atom` — which is what lets a contract clause name a measure of a place on either side of its comparison [MSR-5]; every other position keeps [GRAM-9]'s one-operation-over-two-atoms shape and a nested call is still bound by its own `let`.

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
Representation change is the single explicit op `cvt::<Src, Dst>(x)`.
Totality is decided by value-preservation, not bit-width: `cvt` returns `own Dst` where every value of Src is exactly representable in Dst, and `own Result<Dst, NarrowError>` for every other distinct numeric pair; it never rounds, truncates, or saturates.
The exact partition and per-value semantics are [OP-6].
Deliberate rounding is a separate DEFERRED float-round op family, never `cvt`.

[TYPE-5] Statement-local typing; boundary-explicit facts.
A `let` binder's mode and type are derived, never written: exactly the mode and type its selected right-hand side produces — an `ordinary_let_rhs` from its expression, which is always self-typed (operands are typed atoms, calls are typed by their [FN-1]/[OP-1]/[SYS-2] signatures, literals carry mandatory suffixes [FORM-5], constructions name their nominal and, when that nominal is generic, write its arguments); a `propagate_let_rhs` from the propagated Ok payload [ERR-3]; a `replace_let_rhs` at mode `own` from its target place's final selected type [SET-2]; a `value_match` or `value_if` from the derived common delivery type [GIVE-1], whose delivering `give`s are inside the same `let_stmt`, so the derivation stays statement-local.
This is unique reconstruction, not inference: no binder's type depends on a later statement, an expected type, or any use site, and no two derivations can disagree [FORM-1].
One form is excluded rather than reconstructed: a body `let` may not annotate a borrow with a region its right-hand side did not name, stating a destination the right-hand side satisfies by outlives [OWN-4] rather than equals, and a derived type is always the region the right-hand side itself produces.
Call sites state explicitly exactly what their callee class requires: type and const arguments for user generics [FN-2]; the region arguments [FORM-8] leaves undetermined for a user generic or a system operation [SYS-2]; and, for exactly the retained-argument table operations — `cvt` and `reinterpret` (type pairs [OP-6, OP-8]), `array_new` (element type and const length [CONST-1]), `arena_new` (region and element type), `buffer_fits` and `buffer_vacant` (element type [OP-1, OP-9]), and `finf`/`fnan` (result type) — the written arguments their rows fix, because no operand can supply them.
A `construct` of a generic nominal states that nominal's type and const arguments on the same ground and in every position, mandatorily: the source nominals under [FN-2], and the prelude generic nominals `Option<T>` and `Result<T, E>` through their variant constructors `None`, `Some`, `Ok`, and `Err`.
A nullary `None()` has no operand to supply anything, and construction never consults an expected nominal type [TYPE-6], so the written arguments are the only supply there is; their absence, or a count other than the named nominal's parameter list, is a hard error citing TYPE-5 at the complete `construct`.
The non-generic prelude nominals — `Bool`, `Overflow`, `DivError`, `NarrowError` — have no parameters and write nothing.
Every other table operation carries no written argument and derives its selected type from its operands [OP-2]; a written argument there is a hard error citing OP-1.
Argument types match declared parameter types exactly.
After [SET-1] derives a writable target place of type T, the right-hand side of `set p = e;` must produce exactly `own T`; there is no mode coercion, type conversion, or target-selected operation overload.
After the TYPE-7 implicit-read exclusivity below, a different right-hand-side mode or type is a hard error citing TYPE-5 at the complete `expr` child of the `set_stmt`, carrying expected `own T` and the actual mode and type.
After [SET-2] derives a writable affine target place of type T, the right-hand side of `let x = replace p = e;` receives this same exact-`own T` judgment, located at the complete `expr` child of the `replace_let_rhs`.
Redundant-explicit facts remain mandatory at every trust boundary — signatures with full modes, types, and effect rows [FN-1], construction field names [GRAM-8], match binders [GRAM-10], call argument names [GRAM-11] — and are deleted exactly where reconstruction is unique and no transposition risk exists.
A written region is such a fact only where it relates two positions or names a region the caller must choose; [FORM-8] deletes it everywhere else, because a region determined by its own position carries no fact a reader can check and no transposition it can catch.

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
| REGIONID | `region_params` and a named `region_stmt` | every written REGIONID in `type`, `mode`, `targ`, arena-allocation effects, and `borrow_expr` [FORM-8] |
| LABEL | an optional LABEL written by `loop_stmt` or `for_stmt` | an optional LABEL written by `break_stmt` |
| invariant IDENT | names written by `header_invariant` and `invariant_stmt` | the bare-IDENT source of `proof_use` |

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
A `for_binding` binder becomes visible after its complete `for_binding`, including both endpoint atoms, through the remaining `header_invariant` clauses and the counted body; it is not visible in either endpoint.
An ordinary or counted loop label, when written, and a local region are visible only in their respective bodies; a counted label is not visible in the binding or invariant header.
A loop label is an optional lexical name, never the identity of the loop: every `loop_stmt` and `for_stmt` has one distinct compiler-owned structural loop identity whether or not it writes a LABEL.
An unlabeled `break;` must be lexically inside at least one ordinary or counted loop and resolves to the nearest such enclosing loop.
A labeled `break @name;` performs the ordinary LABEL-domain lookup below and may therefore resolve past one or more inner loops to an enclosing loop carrying that spelling.
The resolved loop's structural identity, not a LABEL declaration, is the target retained by the semantic checker.
A `header_invariant` name is a proof-only declaration in a separate invariant-name domain.
All names in one header must be distinct; none is visible in the header itself or before the loop, and after the complete header all become visible simultaneously throughout that loop body only.
An `invariant_stmt` name becomes visible only after its complete statement through the remainder of its lexical block and nested blocks.
An invariant name never denotes a runtime value, place, ownership object, label, or callable, and it is referenced only by the bare-IDENT source alternative of `proof_use` under [PRF-1].
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
The right-hand side is then checked under [TYPE-5] and evaluated under its ordinary expression, ownership, effect, and partial-operation domain rules.
The checker analyzes the normal continuation of `e` and re-establishes there that the same target root is live and that the resolved target remains writable under the resulting loan state.
If the right-hand side moved the target root, the commit is a later write of a dead root under OWN-1.
If it created or changed a loan that conflicts with the commit, OWN-5 rejects the commit.
This is a static acceptance check: at runtime every target component is evaluated exactly once before `e`, and lowering carries the resulting target address and offset values across `e` rather than evaluating source again.
No root-liveness or writability fact from before the right-hand side bypasses the post-state check.

On successful revalidation, assignment performs exactly one write of the resulting copy value into `p`.
The previous copy value ceases to occupy `p` and requires no drop, release, finalizer, or cleanup edge [STOR-3].
The new value occupies the same place and the target root remains live.
The store occurs only after right-hand-side evaluation completes; until that commit point the target retains its previous value.
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
The exchange commits only after right-hand-side evaluation completes; before that point x is uninitialized and the target retains its previous value.
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
`const-reject` is disjoint from runtime proof-required exact arithmetic: it never creates an [ENT-6] operation obligation or admits a `.defined` spelling, an accepted const-expression executes no runtime check, and a const-expression contributes no runtime effect.
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

[OWN-2] Modes: `own` (owned), `&` (shared borrow), `&uniq` (exclusive borrow); a borrow mode carries one region, written `&'r` or `&uniq 'r` where [FORM-8] writes it and `&` or `&uniq` where [FORM-8] elides it.
The mode itself is always written.

[OWN-3] Regions are lexical.
A `region_stmt` introduces one local region, named `region 'r { ... }` or unnamed `region { ... }`, and every `loop_stmt` or `for_stmt` body introduces one further unnamed local region whose block is that body [OWN-11]; `region_params` introduce the caller-supplied regions, and each unnamed declaration position introduces one further caller-supplied region [FORM-8].
Region identifiers are unique within a function (parameters included); an unnamed region has no identifier and is distinct from every other region of that function.
Outlives-or-equals is the total reflexive relation: `'a` outlives-or-equals `'b` iff `'a = 'b`, or `'a`'s block strictly encloses `'b`'s block, or `'a` is caller-supplied and `'b` is local.
Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject).

[OWN-4] A borrow `&'a p` / `&uniq 'a p` is live exactly until the end of `'a`'s block (named-region liveness).
It may be stored into a destination of declared region `'b`, passed to a parameter of region `'b`, or returned as `rtype` region `'b`, only if `'a` outlives-or-equals `'b`.

[OWN-5] Resolved-place exclusivity.
While `&uniq 'a p` is live and its holder is not suspended [OWN-6]: no place overlapping resolved(`p`) may be read, written, moved, or borrowed, except reads/writes through that borrow's holder and except the creation of a statement-scoped child reborrow, an arm-scoped child reborrow, a candidate-position child reborrow, or a returned reborrow of that holder [OWN-6, OWN-13, OWN-14].
While a holder is suspended (a live statement-scoped child, arm-scoped child, candidate-position child, or returned reborrow of it exists), its own read/write allowance is withdrawn: no read, write, move, copy, `set` commit, or call-transfer through it is admitted until its last child ends; a `&uniq` holder suspended by candidate-position child creation does not resume because the child loan may survive in the bound call result [OWN-6].
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
Under this specification's named-region liveness, moving or returning a descriptor neither shortens nor extends the shared loan established by its source.

[OWN-6] Holder, resolution, and statement-scoped child reborrow.
The holder of a borrow is the binding its `borrow_expr` initializes (a borrow not bound by `let` is a call-scoped temporary, live until the end of the enclosing statement). resolved(place) rewrites a place rooted at a holder binding to the borrowed place plus the appended suffix, recursively.
All OWN-5/OWN-7 judgments use resolved places.
A statement-scoped child reborrow is the written form `&uniq 'c` or `&'c` over `deref(h)` followed by any written suffix chain, occurring as an argument atom of a `call` expression [GRAM-9], admitted only when: the receiving call's result mode is `own` or `unit`, never a borrow — except in the receiving call's provenance-candidate position, where a borrow result is admitted; `'c` is a locally-introduced region [OWN-3] whose block does not extend beyond the enclosing statement, and a caller-supplied region parameter is not admitted — except in the provenance-candidate position, where `'c` is any live region that resolved(`h`)'s region outlives-or-equals, caller-supplied included; the eligible holder `h` is a function parameter or a `let`-bound borrow, never a `match` binder; and a `uniq` child has a `uniq` parent, while a `shared` child is admitted from either [OWN-5]. resolved(child) = resolved(`h`) ++ suffix.
Creating a child suspends `h` for the enclosing statement [OWN-5]; while a holder is suspended by this statement-scoped creation, the sole operation admitted through a place overlapping resolved(`h`) is creating a further sibling child, siblings judged by OWN-7 with any overlapping pair containing a `uniq` child an error, and `h` resumes at the end of the statement after its last child ends.
Creating a candidate-position child through a `&uniq` holder suspends that holder for the remainder of its life; there is no statement-end resumption, because the child's loan may survive in the bound call result.
A shared holder needs no suspension: it admits no write through itself.
A child is never bound, returned, `give`n, stored, or the whole call result, and its `'c` cannot outlive the statement, so no borrow derived from a child outlives its statement; with borrow-free storage [STOR-5] the child is non-escaping.

A `let` whose ordinary right-hand side is a user call with borrow-mode result is a borrow holder rooted at the callee's provenance candidate [FN-1], and every accepted callee has one or has none.
resolved(result holder) = the candidate actual's complete resolved place, even when the callee delivered a narrower suffix of it; the holder's borrow is otherwise ordinary — OWN-4 liveness in the substituted result region, OWN-5 exclusivity, OWN-6 child admission, OWN-14 returned reborrow.
Nothing here narrows FN-1: the caller still judges the call by the signature alone.
A borrow-mode call result with no candidate is rooted in named `const` storage [FN-1, CONST-2], which no accepted write or unique borrow reaches [OWN-5, OWN-7]; its holder borrows no caller place and conflicts with nothing.

Bound children, result-carrying children (reference-result provenance), `uniq`-to-`shared` downgrade, `match`-binder parents, and written grandchild chains through a bound direct reborrow are DEFERRED with recorded delta; every written reborrow form outside this argument-atom position is dispositioned by [OWN-14], and the derived match-payload binder is [OWN-13]'s arm-scoped child reborrow.

[OWN-7] Overlap: resolved `p` overlaps resolved `q` iff one is a prefix of the other.
Two subscripted places with the same resolved base overlap iff their offsets are not both literals with unequal values.
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
Inside such a body a `borrow_expr` may denote only regions introduced inside that same loop body — the body's own region, or a `region_stmt` inside it — whether it writes the name or elides it [FORM-8], and a binding declared outside that body may not be moved inside it (copies exempt).
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
The accepted [OP-9] judgment retains a numeric upper bound for the source length at that allocation site; target qualification multiplies that bound by the actual target stride using checked mathematical arithmetic and requires the result to fit both the allocator-parameter and address-index domains before lowering the operation.
At this target stage, the exact SSA result of `len(buffer)` additionally carries the selected target's runtime-allocation byte maximum divided by that buffer's actual element stride, because every materialized buffer already satisfies the successful-allocation representation invariant.
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
An `infix_op` or `compare_op` token resolves to its exactly spelled operation by the operator table row; infix resolution consults no name domain, and an operator token is never a declaration, callee IDENT, or OPNAME.
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

[OP-4] A subscript `p[i]` selects one element place of an indexable base: the base place `p`'s final selected type must be `array<T, N>`, `slice<'r, T>`, or `buffer<T>`, and the subscripted place's selected type is exactly that element type T — derived from the base place's already-fixed type [TYPE-5] — written where the binding carries an annotation, derived at a body `let` — by the same declared-type selection that types a field suffix, never from expected type or cross-statement inference; a subscript whose base's final selected type is not one of the three indexable types is a hard error citing OP-4 at that subscript's `psuffix` node.
The subscript carries the bounds obligation `i < len(p)` [ENT-6].
A discharged subscript reads or writes with no runtime bounds check in every build mode, and its checked-program disposition records the discharging derivation [DIAG-2].
A subscript whose current ProofContext does not discharge the obligation is a compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying the residual obligation rendered exactly per [ENT-6], and publishes no checked program.
Its mechanical fix is a dominating branch establishing the residual [ENT-3], a proved header or local invariant [INV-1], an invariant carrying sufficient [PRF-1] uses, or a verified callee relation [FN-9].
Discharge is a deterministic checker derivation [ENT-1]; a solver result never participates.
A `buffer<T>` obligation is over the runtime length term.
The offset atom has exact value mode and type `own u64`; after the [TYPE-7] implicit-read exclusivity, any other offset mode or type is a hard error citing OP-4 at the offset `atom` node, with `SourceCoordinate` equal to that atom's complete checked half-open source extent.
A subscript in a [SET-1] target forms the selected place without reading its stored value; its base and offset are evaluated during target evaluation, and its discharge judgment is identical in target position.
A successful bounds judgment neither narrows nor authorizes narrowing the offset or its scaled byte offset; target address formation additionally obeys [STOR-6].
System range calls carry their own static [SYS-8] obligations through the same [ENT-6] framework; no operation-internal range check is retained.

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

[OP-7] Operation-name convention (regularity, W1-predictable).
An arithmetic, logic, bit, or compare op carries a domain prefix — `i` (integer), `f` (float), `b` (Bool logic), or `e` (tag-only enum comparison, including `Bool`) — whether or not a cross-domain twin exists; the structural ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix.
The integer arithmetic and integer comparison symbols of [GRAM-5] are the one prefix-free operation class: each is an integer-only table row, so `+` and `<` never denote a float or enum operation, and `fadd.strict`, `feq`, and `eeq` keep their prefixed names.
`Bool` participates in the `b` family for boolean logic and the `e` family for tag-only equality; the operation name, not operand inference, selects the family.
A respelled operation's token is its one constant spelling under the same one-spelling-per-operation discipline.
Bare infix and dotless named integer spellings are proof-required exact operations; `.defined` is the distinct total Bool-valued domain query, not a result mode and not an execution of the partial primitive.
The total value-result policies remain `.wrap`, `.checked`, and `.sat` where [OP-1] lists them, and float `.strict` is unchanged.
Signedness-parametric lowering keyed on the operand-derived selected type [OP-2] (`ishr` is `ashr` for signed T and `lshr` for unsigned T; `imin` is `smin` or `umin`) is the same discipline as the `<` = `slt`/`ult` row, not overloading.
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
Each is accepted only when [ENT-6] discharges that exact goal; its sole normalized component is the defining comparison above, which may supply an alternate L0 derivation of the same root.
The root does not project a new general L0 fact in the other direction.
A refuted or unproved goal is a static OP-9 rejection; a contradictory state discharges it under [ENT-4].
When n comes from runtime input, it remains an ordinary symbolic term; only an enumerated fact constructor such as a selected real branch, a proved invariant target (including one checked by [PRF-1]), or a verified postcondition may discharge this goal [SCOPE-2, ENT-3, ENT-6].
No written conclusion alone, runtime multiplication guard, or fallback is retained.

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

[FN-1] A concrete function's callable boundary states everything ordinary callers need: parameter modes and types, the named result's mode and type, one formal-path state-effect row, its region parameters and the unnamed regions of its remaining region positions [FORM-8], the ordered [FN-8] requirement GoalTemplates, the ordered verified [FN-9] normal-result RelationTemplates, one compiler-derived result-state routing summary, and one compiler-derived target summary.
The result binder's spelling is mandatory but ignored by callable-signature equality and denotes no runtime storage.
The written templates are checked interface propositions rather than trusted declarations; a caller consults only their verified finite summaries and never a callee body.
The written effect paths state which parameter-supplied state the function observes or changes. The checker derives the exact same set from body accesses, direct system contracts, releases, and calls and checks it in both directions under [EFF-2].
The result-state routing summary records, for each ordinary owned state leaf the result may carry, whether that value is fresh or is the same value supplied by one or more formal parameter leaves. It is derived from existing move, construction, match, return, and ownership flow; it adds no source syntax, identity, parent relation, permission, or runtime field. A caller uses it only to preserve those existing value identities when a later effect or compiler-derived release acts through the returned owner. A result with no state leaf has the empty summary.
The target summary states `never-suspends` or `may-suspend` and, for each reachable suspending action, the applicable `result-ready` components, `loan-released(formal path)` facts, and `terminal`; a release with no writer result has no `result-ready` milestone.
That summary is derived from exact system contracts and the finite concrete call graph, never written, inferred from a spelling, or weakened by a declaration. It describes suspension and ownership handoff only; it grants no access and supplies no concurrency or alias judgment.
Strengthening a requirement GoalTemplate or RelationTemplate is a caller-visible interface change.
A generic function carries the same boundary with its written type and const parameters, and each concrete [FN-2] instance substitutes them before its calls and body are re-checked.
A `fn_sig` has neither kind of template.
Function-signature visibility is the [TYPE-6] table.
Every explicit `return e;` must produce exactly the enclosing function's `result_binding` `rtype`; there is no result-mode or result-type conversion [TYPE-4].
The implicit-read case already owned by [TYPE-7] is exclusive: when `e` uses a borrow-mode or box/arena binding where its referent value would be required by the written `rtype`, that use is rejected citing TYPE-7 and FN-1 forms no candidate.
Every other return mode or type mismatch is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child.
FN-9 adds a stricter result and return-expression shape only for a function that declares an `ensures_clause`; a function with none retains every return form admitted here.

For a function whose written result is `own slice<'r, T>`, the written signature also determines one return-origin ceiling without additional syntax.
The ceiling contains `immutable-const` and the formal-slice origin of every parameter whose mode and type are exactly `own slice<'r, T>` denoting that same formal region and element type; an elided parameter region denotes a distinct region [FORM-8] and therefore never supplies the result.
No parameter with a different mode, type, element type, or formal region is a supplier.
In particular a borrow-mode parameter and an `arena<'r, U>` parameter are not implicit slice suppliers.
Every explicit `return e;` producing that written result must have an [OWN-5] origin set contained in the ceiling.
Failure is a hard error citing FN-1 at the `return_stmt` node, with `SourceCoordinate` equal to the complete checked half-open source extent of its selected `expr` child and the restructuring `accept an exact direct input slice in the result region or keep the newly formed view in its caller; do not return a view of raw callee storage`.
OWN-10 independently rejects a returned origin whose storage is too short-lived.

A function whose written result mode is `&'d` or `&uniq 'd` and whose direct result type is `slice<'r, T>` is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor`.
This specification has no signature summary that carries both the returned descriptor's source-place provenance and the underlying slice value's complete origin set.
This rejection does not change any other returned-borrow judgment.
A function whose result mode is `&'b` or `&uniq 'b` determines the result's provenance from its written parameters alone: a parameter is a provenance candidate iff its mode is a borrow of the result's kind in the result's formal region `'b` [OWN-6].
Because a candidate shares `'b`, [FORM-8] writes `'b` at both positions; a function with no candidate writes `'b` at its result alone and its caller supplies that region.
Exactly one candidate is the result's debtor, and zero candidates is legal — OWN-10 admits no `'b`-region borrow rooted in callee-local storage, so the only remaining source is named `const` storage, whose immutable program-lifetime extent needs no caller loan [CONST-2].
The provenance judgment applies to a result whose written type is region-free; a region-bearing result type is rejected before it — a direct slice by this rule's slice sentence and an arena, in either result mode, by [STOR-4].
Two or more candidates, a same-region parameter of the other borrow kind, or any parameter whose type carries `'b` leaves the source undetermined and is a hard error citing FN-1 at the complete `rtype`, with `SourceCoordinate` equal to that production's complete checked half-open source extent and the restructuring `give the source parameter its own region so exactly one parameter shares the result's region and kind, or return the decision as a value and let the caller borrow from the source it names`.
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

[FN-2] Function and nominal generics are monomorphization-only; type and const instantiation arguments are always explicit and region arguments are written exactly where [FORM-8] writes them; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.
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
After each signature's independently applicable EFF-1 judgment and the bound function declaration's EFF-2 judgment succeed, an effect row normalizes to three components: the set of declared read paths, the set of declared write paths, and the allocation set whose members are `heap` and each alpha-mapped `arena` region; `pure` is three empty components.
An effect path uses its root parameter's zero-based ordinal followed by its static source-struct field ordinals. Parameter and field spellings do not create signature identity.
Equality requires all three components to be equal.
A `fn_sig` has no body and no compiler-derived release, so it declares these components without an EFF-2 judgment of its own; the bound `fn_decl` must exhibit exactly the member's declared row under [EFF-2], including a path the bound function contributes only through release.
The compiler-derived target summary is not a writer declaration and does not participate in source contract equality. After a conformance selects a concrete bound function, ordinary closed-world propagation uses that function's derived summary at every call.
Source occurrence order and repeated occurrences do not affect this equality, but no path or allocation component may be omitted or added; there is no effect subtyping or semantic implication.
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
Each successfully discharged `(contract-law node, concrete-conformance node)` pair contributes exactly one checked law fact and its base diagnostic derivation to the checked program [DIAG-2].
They reference the conformance, contract law, bound function, operation row, concrete domain, law, and optional identity; no pair is omitted, shared, or deduplicated.
Source acceptance is the result of that originating semantic check itself.
The checked fact may be consumed later, while its diagnostic derivation only explains that originating semantic decision and grants no independent authority.

A checked law fact may affect optimization only through a specification-fixed transformation rule that binds the exact checked-program instance, target, backend, proposition, and authorized consequence.
That rule checks whether the current transformation site has the required IR and target shape, then consumes the law fact produced by the source semantic check; it does not independently rederive the contract/member/body/table/identity relation.
For a gated law, the checked fact also carries the exact ledger-entry identity and scope required by [LEDGER-1].
Absence of the checked fact, an inapplicable transformation site, or resource failure leaves source acceptance, semantic identity, explicit checks, and facts-off lowering unchanged.
A pre-approved opaque gated-family signature may separately contribute a checked law fact through its soundness-obligation ledger [LEDGER-1], but that proposition is not a source `conform` discharge and reaches an optimizer only through the same fixed transformation-applicability boundary.
Additional operation rows and other complete-domain source proof calculi are DEFERRED specification additions.
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
When such a form is proposed, monomorphized direct calls rather than function-pointer or dictionary dispatch remain the performance candidate to test; this specification does not define that unavailable mechanism.

[FN-6] Recursion is permitted.
Polymorphic recursion is rejected by a syntactic rule: in any call cycle among generic functions, every call instantiates the callee at exactly the caller's own type parameters.
This criterion is DELIBERATELY stronger than finiteness requires (it rejects some finite permutation cycles): predictable, locally explainable rejection per OWN-8's reject-and-restructure posture; the diagnostic must name the cycle and the restructuring.
Rejection-rate measurement is a registered experiment.

[FN-7] Exactly one top-level `fn_decl` named `main` must exist in the compilation unit.
That declaration is the unit's sole entry and must carry the exact fixed `command` program-kind marker.
It is nongeneric, declares no region parameters, and has no `contract_block`.
Its mandatory result binder is writer-named and its written result is exactly `own ExitStatus`.
Its written effect row is any subset of `reads` and `writes` paths rooted in its own labelled inputs and `allocates(heap)`, in [EFF-1] canonical order; `pure` is the empty subset and no arena allocation is admitted.
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

[FN-8] Every non-entry source `fn_decl`, generic or nongeneric, may carry one optional `contract_block`; [FN-7] forbids it on main and `fn_sig` has no such production.
A present block must contain at least one `requires_clause` or `ensures_clause`; an empty or define-only block is an FN-8 rejection at `contract_block`.
Grammar fixes all definitions before all requirements and all requirements before all postconditions.

The definition scope initially contains the function parameters, named consts, and live type and const parameters, then each earlier definition after its complete initializer.
Every definition and clause expression must consist only of non-consuming datums and operation-table forms that are pure and total for every value in their selected operand domain.
User and system calls, construction, move, borrow, subscript, mutation, control flow, allocation, and every proof-required exact or otherwise partial operation are inadmissible even when another clause states their domain, with one exception: exact addition, subtraction, and multiplication are admitted and are read as operations over the mathematical integers rather than as evaluations, exactly as an `affine_expr` is [INV-1]. A clause is erased before lowering and evaluates nothing, so a row whose meaning is total over the mathematical integers states a relation where it would otherwise request an operation, and no domain obligation arises to discharge. Exact division, remainder, negation, absolute value, and the shifts stay inadmissible: each has an input its own relation cannot state its way out of, so admitting it would place a partial operation where nothing discharges it.
The corresponding `.defined` queries are total and admissible.
Each definition produces an own copy value, follows ordinary typing and no-shadowing, and is erased by recursive alpha-expansion into every later clause; no definition is evaluated, snapshotted, lowered, or visible in the body.

Each requires expression is one `clause_expr` [GRAM-5, MSR-5], has exact mode and type `own Bool` under [OP-5], and independently forms one finite typed GoalTemplate after definition expansion.
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
Main is not source-callable [FN-7].

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

[FN-9] Each `ensures_clause` in a [FN-8] `contract_block` declares one independent verified normal-return relation.
It is neither an executable epilogue nor a trusted assertion; no contract definition or clause contributes an effect, runtime operation, storage slot, or runtime report.

An unrouted clause is admitted only when the written result is `own T` and T is one [ENT-2] fragment integer after concrete [FN-2] substitution.
Its symbolic whole-result datum is the header `result_binding`.
A routed clause is admitted only as exact `when Ok(value: r):` for written result `own Result<T,E>`, where T is a fragment integer and r is that clause's fresh symbolic payload datum; `Ok` and `value` retain their PRE-1 identities.
Route owner, variant, field, and freshness admission precedes resolution of that clause expression [GRAM-10, TYPE-6].
The header whole-Result binder is unavailable in a routed clause.
Borrow-mode, unit, float, aggregate, nested-payload, whole-Result, non-Ok, and every other shape remains a legal ordinary result but cannot supply a relation datum in this version.
Omitting Err routes means Err exits are unselected, not unreachable.

After recursively alpha-expanding every shared `contract_define`, the clause expression must have exact type `own Bool` and its root must be exactly one `compare_op` — `==`, `!=`, `<`, `<=`, `>`, or `>=` [GRAM-5].
Both operands must be the clause's symbolic result datum, a parameter datum with field and `deref` projections, a named const, a typed integer literal, or `len(P)` for an admitted formal place P [MSR-5]; at least one operand contains the result datum.
A `len(P)` operand whose place P is rooted at a `&uniq` parameter is inadmissible in an `ensures_clause` and is a hard error citing MSR-3 at that clause.
No proof-required exact operation, computed arithmetic result, subscript, occurrence-local evaluated-value datum, Boolean connective, nested result projection, or body local becomes a relation term.
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
The relation is queried once in the current ProofContext at that return.
Every query must discharge; the first clause/return failure rejects with no runtime fallback.

Postcondition verification has no summary fixed point.
Form the concrete ordinary-call graph, its SCCs, and the callee-before-caller condensation.
While verifying a component, all same-component S12 summaries are unavailable; previously completed callee components remain available.
Only after every relation of every inhabited instance in the component succeeds are all its relation summaries published atomically; any failure publishes none.
Uninhabited instances contribute no summary.
Declaration or worklist order and iteration cannot change the result.

For one ordinary call c, `A0(c)` means resolution, concrete instantiation, named arguments, exact types, borrow feasibility, every actual-expression obligation, exact formal substitution, and success of every FN-8 requirement have all occurred in that order at the same pre-transfer point.
Failure forms no postcondition candidate.
For one relation q, `M(c,q)` holds only when q's route matches that exact establishment event, result and referenced formals substitute independently to live [ENT-2] terms or constants after ordinary kills, and no referenced actual is represented only by an occurrence-local evaluated-value datum.
A discarded or nested result, stored or propagated whole outcome, unsupported or unselected route, killed support, or nonterm actual makes only that M false.

Subject to A0 and M, failure-atomic scratch establishes q after transfer, consumes, borrow commits, callee-effect kills, and target kills.
Every establishment retains the selected-return proof plus all actual-obligation and requirement parents from A0.

All matching verified relations are established together on the admitted result route.
An unrouted fragment result establishes onto the fresh binding of a direct ordinary-let call.
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
A `requires_clause` and an `ensures_clause` take a `clause_expr` [GRAM-2, GRAM-5], whose operands are each an `atom`, a `call`, or a `construct`.
The operand set is the whole of this rule: [GRAM-5]'s `atom` has no `call` alternative, so before this version a measure of a place derived nowhere in a clause and `len(source) <= len(out)` was a GRAM-5 parse rejection at the comparison, while the same fact written through one `contract_define` per operand was admitted — one semantics with two spellings, one of which cost a definition per measure.
A clause is judged by exactly the [OP-5] condition [FN-8] and [FN-9] already apply: the root has exact value mode and type `own Bool`, and every operand is a non-consuming datum or an operation-table form pure and total over its selected operand domain.
This rule adds no route, no fact source, and no proof authority; it adds spellings the existing admissions already accept.

The measure formers are table data over the measured types, with one row in this version: `len(P)`, of fragment type u64, admitted for exactly the places [ENT-2] clause (b) admits a length term for.
A measure former is written as an ordinary `call` whose one `atom_list` operand is that place; a written type argument, a `fieldinit_list`, or a second operand is the ordinary [OP-1] rejection.
A clause operand that is neither an [ENT-2] term nor a constant stays an ordinary pure total operand contributing no L0 projection; clause position makes nothing a term.

The `affine_factor` production of [GRAM-4] is not widened, so a `header_invariant`, an `invariant_stmt`, and a `proof_use` keep exactly [INV-1] 3109-3113's atom admission.
A measure term is now an affine atom, which is the admission the previous version recorded as deferred and did not take.
The affine domain carries one atom per measured place, minted on first use at its full u64 range and identified by that place's root binding, and the L0-to-affine index ranges over measure terms, so the bounds a creation or a verified contract established on `len(P)` tighten that atom exactly as an ordinary binding's bounds tighten its own image.
The atom is stable while the object is: a measure is fixed at its object's creation and an element write never moves it [ENT-5], so only a write to the root binding removes it, which mints a new unknown for the next read.
A structural join keeps the atom where every input agrees and drops it otherwise; a measure is not arithmetic-updated, so there is no spread for a join delta to stand for, and inputs that disagree disagree because some branch replaced the object.
A measure of a projected place is not this atom and remains an ordinary clause operand.

[CALL-4] Contract vocabulary, the result ordinal, the routes, and where the relations land.
The clause operands of [FN-9] are terms [MSR-5], so a measure over an admitted formal place is an operand with no per-family admission, and so is one over an admitted result place.
A `fn_decl` declares exactly one `result_binding` [GRAM-2], so the ordinal a route applies to is that one result, the route names no ordinal, and no ordinal binder is written; a declaration with more than one result is not a form of this version and neither is a destructuring binder over one.
The result datum's type is a fragment integer after concrete [FN-2] substitution [FN-9], and the measure table gives a fragment integer no row [MSR-5], so no measure of a result is an admitted operand in this version.
DEFERRED: a result of measured type, a measure over a result place, and a route over any variant of any returned enum type; their delta is numbered rules +0 and grammar productions +0, and each is an admission widening of [FN-9] rather than a new judgment.

The destinations are exactly [ENT-3.S12] 2822-2837's, and a relation reaches a caller only there; [CALL-6] fixes the point at which each is instantiated and the point at which each is established.

## 9. Effects (unified-state revision)

[EFF-1] Row grammar: the `effects` and `effect` productions of the fence below, in exactly this canonical order (reads, writes, allocates).

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" effect_path ("," effect_path)* ")"
        | "writes" "(" effect_path ("," effect_path)* ")"
        | "allocates" "(" ("heap" | "arena" REGIONID)+ ")"
effect_path := IDENT ("." IDENT)*
```

A category appears at most once in one row.
`pure` is the unique spelling of the empty row.
Frame residency (STOR-1) is not an allocation by definition.
The spellings `external`, `blocks`, `memory`, `world`, and `capability` are not grammar atoms, effects, retired spellings, or reserved words. They satisfy IDENT wherever any other lowercase identifier does.

Every `effect_path` is rooted at one formal value parameter of the same callable. Each suffix selects one statically known source-struct field from the preceding type. A root resolving to a local, result binder, unrelated declaration, or non-parameter declaration is an EFF-1 rejection. An unknown field, an enum payload, a dynamic subscript, a dereference spelling, and every other place form are outside this candidate grammar. A bare parameter names the complete state that parameter supplies; a field path names only that structural substate.

For a borrow parameter, its effect path names the borrowed referent rather than the local reference representation. For a direct `slice<'r, T>` parameter, it names the viewed backing state rather than the descriptor. For an `own` parameter, the path names the incoming owned state. Merely moving, returning, or structurally repacking that value does not observe or change it; an operation which reads or changes its contents exhibits the corresponding path. A REGIONID never names effect identity: regions state loan liveness and outlives relations only.

The row describes observations and changes of ordinary Whitefoot state and allocation. It does not distinguish memory from outside state and does not describe a host scheduling mechanism. Opaque system resources, buffers, aggregates, factories, permits, clocks, and Sources all use the same path, exactness, call-substitution, and ownership rules. No type or path carries a writer-visible capability category.
`reads(path)` means the operation observes that state. `writes(path)` means the operation replaces or advances that state. They remain independent exact facts: an operation which observes prior state while changing it names the path in both categories, while a complete overwrite need only write it.
Whether a target uses a native completion queue, readiness, polling, a bounded blocking helper, an interrupt, or inline completion is target data [QUAL-1], not a source effect.

[EFF-2] A concrete function declaration exhibits the union of exactly two contributions: its body-syntactic contribution and its release contribution.
The body-syntactic contribution is syntactic over the complete function body: it exhibits reads, writes, and allocations from the resolved accesses, calls, and allocation operations the body uses.
Proof-required exact integer operations, integer domain queries, proved allocation operations, and proved system-range operations contribute no runtime fallback and no extra effect, because source acceptance precedes lowering.
A bare operator inside a `const` [CONST-1] is const evaluation under `const-reject`: an unproved domain rejects the declaration during compilation and contributes nothing to any effect row or runtime path.
An optional `contract_block` consists only of erased definitions and proof clauses [FN-8, FN-9]; it contributes no read, write, or allocation category.
An [FN-8] uninhabited instance still derives and checks this contribution from its complete source body; the unreachable lowering stub never narrows its written callable row.
The release contribution is defined below and has no syntactic occurrence anywhere in the declaration.
A `for_stmt` endpoint and body contribute their ordinary source occurrences under these same clauses, and its body-exit cleanup contributes under the release rule below.
Its compiler-owned captures, binder initialization, header comparison, and representable hidden update contribute no read, write, or allocation effect.
Function-body attribution and call-boundary projection are separate judgments.

While one function body is checked, every exhibited read or write is attributed after ordinary place resolution, holder resolution, and [OWN-5] slice-view provenance.
An access to a borrowed referent, direct slice backing, or incoming owned state contributes the most precise formal-rooted static struct path [EFF-1] admits for that state. A dynamic element or range access maps to its nearest statically nameable enclosing path because [EFF-1] admits no dynamic selector.
A read through a shared, exclusive, or owned parameter may exhibit `reads(path)`. A write may exhibit `writes(path)` only when ordinary ownership already grants exclusive or owned access to that state. The effect path grants no permission, changes no loan extent, and cannot narrow a borrow of a whole aggregate to one field.
A named const root and `immutable-const` contribute no read effect because their state is permanently fixed [CONST-2]. Moving, returning, or repacking an incoming owner without inspecting or changing it contributes no effect. A fresh local own binding contributes no enclosing read or write effect, even when reached through a local borrow, local slice, or later local move.

A direct `slice<'r, T>` parameter names its viewed backing state rather than its descriptor. Reading through it contributes `reads(parameter)`; a slice derived from an incoming buffer or slice parameter retains that formal-rooted origin, and a multi-origin slice contributes the deduplicated union of every formal-rooted origin. The descriptor's own mode region still governs its loan, but no lifetime spelling enters an effect row.
Binding, moving, passing, returning, borrowing, reborrowing, and slicing preserve the existing resolved place identity. This is the same identity tracking already required by ownership and move checking; EFF-2 adds no parent link, result ancestry, resource root, or second provenance system.

At a user or system call, each callee effect path selects its root formal's actual argument and appends its static field suffix to that actual's resolved place. Holder resolution then reaches the borrowed referent, and a slice actual projects through its complete [OWN-5] origin set. A projection rooted in one of the current function's formals contributes the corresponding current-function path. A projection rooted only in fresh local state contributes no enclosing effect.
Thus a callee write through a child reborrow of incoming `&uniq` storage reaches the incoming formal path, while the same callee write through fresh local storage frames out. Equal lifetime arguments never merge two suppliers because lifetimes do not participate in this substitution.

Resource-producing calls follow the same rule. For example, `reserve_file(factory: &uniq factory)` exhibits the callee's `writes(factory)` on the caller's `factory`; an open with `permit: move permit` exhibits `writes(permit)` only on that local permit; and later operations on the returned fresh local resource remain local. Creating the permit or resource establishes no hidden child-to-factory ancestry. Any externally visible change to the factory or namespace is the direct effect of the operation that changes that parameter and must appear in that operation's own row [EFF-5].
Framing an action on fresh local state out of the enclosing signature means only that it contributes no formal-rooted boundary path. The checked call still retains its instantiated nonempty effect on that local place. Eliminating it requires the ordinary closed-state, escape, result, release, and observer proof which justifies deleting any stateful call; absence from the enclosing row alone proves none of those facts. A target operation is lowered with its qualified physical side effects intact. The mandatory direct write on the creating factory, namespace, allocator, or permit prevents the enclosing call from becoming `pure` merely because the produced owner stayed local [EFF-5].

The release contribution collects the effects of compiler-derived release.
Under [STOR-3] each type fixes one compiler-derived release action together with that action's state-effect row.
For the function being checked, the release contribution is the union of the effect rows of every release action that may run on any edge of the conservative structural normal-control graph defined in [FN-1].
A release contributes when it may run on at least one such edge; running on only some paths never weakens it, and no path condition, constant evaluation, discharged law, optimizer fact, or backend reachability judgment removes an edge from that graph.
An owner moved or returned on one `match` arm and released on another therefore contributes its release row to the enclosing function, and so does a release derived on only one arm of any other branch, one `give` edge, one propagation edge, or one loop exit.
On each normal edge every owner has exactly one disposition — moved or returned, consumed by an explicit consuming operation, or released by exactly one compiler-derived release action — so one owner contributes at most one release per edge, and an owner consumed on that edge contributes no release there.
Release actions run only on the normal edges fixed by the source control-flow rules.
A release action substitutes its released owner's resolved identity for the type contract's table-local `owner` path. Releasing an incoming owner, including one first moved through local bindings, therefore reaches that incoming formal path; releasing a fresh local owner frames out. A release-derived effect inside a callee belongs to that callee's row and reaches the caller only through the ordinary call-boundary projection of the callee's declared row; it is never attributed to two functions. The release's suspension and milestone summary propagates separately under [FN-1].

This attribution reads only the release rows [STOR-3] fixes, and it does not retrofit memory reclamation into effect rows.
A `box<T>` drop, a `buffer<T>` drop, an `arena<'r, T>` region release, and the absent drop of a `const` item [CONST-2] each carry the empty release row and therefore contribute nothing to any function's exhibited row; only a system resource type whose contract fixes a nonempty release row contributes one.

A [SET-1] commit is one write under this attribution, and a [SET-2] commit is one read and one write of the same target origin.
A shared-holder commit is rejected [OWN-5] and contributes no accepted effect judgment.
Effects exhibited while evaluating the target and right-hand side contribute normally; an accepted target subscript is discharged [OP-4] and contributes no extra effect.
Rows are checked both ways against the exhibited row defined above: undeclared-but-exhibited and declared-but-unexhibited are both errors, and an entry contributed only by the release contribution is checked exactly like one written in the body.
A mismatch involving the release contribution has no offending source occurrence, so it is a hard error citing EFF-2 using `SourceNode` at that function's `effects` node, with `SourceCoordinate` equal to that node's complete checked half-open source extent; the diagnostic additionally renders the parameter or binding whose release contributed the category, and the restructuring `declare the release effects of every resource this function may release, or move the owner out`.
When more than one owner establishes that premise, the reported one follows DIAG-1's implementation-defined deterministic traversal.
A function whose body and release contribution are empty may therefore declare `pure` while carrying an erased contract.

Canonically, a nongeneric function whose only parameter is `own ReadFile` and whose complete body is exactly `return unit;` declares `writes(file)`.
Its compiler-derived release contributes that state write and a `may-suspend` target action on the function-return edge. Declaring `pure` is an undeclared-but-exhibited EFF-2 rejection.
This shape cannot be reduced further: [FN-1] requires the body's normal exit to be unreachable, so a function with an empty body is separately rejected and is not the canonical case.

[EFF-3] A call whose row is `pure` and whose derived target summary is `never-suspends` licenses deduplication and reordering with equal arguments.
Elimination of an unused such call additionally requires a termination proof; v0 provides no termination checker, so unused calls are not eliminated.
The source spelling `pure` excludes state reads, state writes, and allocations; it does not promise termination.
A call that exhibits `writes(path)` may remain observable even when its result is unused. A call on fresh local state retains that instantiated effect even though it frames out of the enclosing signature. No optimization may erase, duplicate, speculate, or reorder either call unless ordinary effect-path overlap, closed-state, escape, ownership, control, result, release, and surviving-observer proofs establish the exact transformation; system state receives no separate effect category or observability tag.

[EFF-4] Accepted source has no writer-reachable abort effect, exception, unwinding edge, or hidden runtime proof fallback.
Every proof failure rejects the source before lowering.
Target qualification failure, unavailable external resources, and a trusted-computing-base failure remain outside the source effect system under [SCOPE-3]; none creates a writer-visible effect spelling or an alternate successful source judgment.

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

[ERR-4] Classification: expected environment and input failures represented by an operation contract are values (`Result`); unproved function, operation-domain, allocation-fit, bounds, system-range, layout, address, and target-domain obligations attached to source execution are source rejections.
Unavailable external resources and trusted-computing-base failures remain outside the source outcome model under [SCOPE-3].
An operation's classification is fixed by its table row and attached static obligations, never by call-site preference.
The overlap permissions and bounded queue/completion protocols of [PAR-1–PAR-3] are implementation permissions over an already accepted sequential program, not source obligations in this version: absence of a complete permission derivation retains sequential lowering and never rejects the source.
If an implementation does select an overlapping or bounded-completion lowering, every premise of that permission and protocol must be discharged before emission; a failed premise cannot be repaired by a runtime check or partially parallel fallback.

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

Supplying each declared standard input is a start-time obligation of the selected target.
When the selected target cannot supply one, start fails before the body is invoked: no source statement executes, no owner comes into existence, no language cleanup runs, and no `ExitStatus` is produced.
A start failure is a target or environment failure.
It is not a source-language rejection [DIAG-1], produces no source result, and never rewrites a source acceptance judgment.

A `command` entry that completes normally returns exactly one `own ExitStatus` [FN-1].
Compiler-derived release for every owner live on that return edge runs before the instance terminates [STOR-3].
The selected target then maps that returned value to the process status exactly.
No other source value, written output, effect, release result, or target condition contributes to that status, and the language defines no second normal status channel.

External start-resource failure and trusted-computing-base termination are outside the returned status, and `ExitStatus` carries normal command status only.

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
If the boundary token is one member of four consecutive actual tokens `IDENT "." IDENT ("("|"::")`, that dotted call-or-targs spelling cites [FORM-3].
Its coordinate is the complete interval from the first IDENT through the second IDENT.
An allowed suffix would already be one maximal OPNAME token, while a field place cannot be called or given targs.
This bounded diagnostic window may include already recognized tokens, performs no operation-table or name lookup, consumes nothing, and does not enlarge recognition's two-token lookahead.
2.
If source-EBNF provenance reaches or would next enter an `atom` occurrence in `atom_list`, `fieldinit`, an `infix` operand, the subscript offset, or either endpoint of a `for_stmt`, and the two actual tokens at the start of that occurrence are `(IDENT, "(")`, `(IDENT, "::")`, `(OPNAME, "(")`, `(OPNAME, "::")`, `(TYPEID, "(")`, or `(TYPEID, "<")`, the rejection cites [GRAM-9]; in an infix-operand occurrence, a two-token start whose second token is an `infix_op` or `compare_op` token — the forbidden nested-infix start — likewise cites [GRAM-9].
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
4. a declaration collision with PRE-1;
5. a declaration collision with an admitted system declaration [SYS-1];
6. a compilation-root duplicate or same-lexical-scope redeclaration; and
7. a nested declaration shadowing a live declaration.

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
Conflict domains use the fixed order lexical-IDENT, nominal-type, constructor, contract, REGIONID, LABEL, invariant.
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

If inventory succeeds, every lexical use admitted by TYPE-6, OP-1, INV-1, or PRF-1 creates one lexical-use event.
The generic-numeric suffix admits a live generic TYPEID parameter; FN-3 and FORM-5, not lexical resolution, later require its numeric bound.
Lexical resolution fixes only the declaration or operation-family target.

The closed declaration-class order is function, named-const, const-generic, value, generic-type, nominal-type, struct-constructor, enum-variant, contract, region, label, invariant, operation-family.
TYPE-6, OP-1, INV-1, and PRF-1 fix each lexical role's ordered admissible subset.
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
| affine IDENT in a `header_invariant` or `invariant_stmt` target | INV-1 |
| affine IDENT in a relation-form `proof_use` source | PRF-1 |
| bare-IDENT `proof_use` invariant source | INV-1 |

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
The name of every `header_invariant` and `invariant_stmt` produces one proof-only invariant declaration record that uses TYPE-6's inventory and scope machinery; [INV-1] owns collision and lookup failure in this domain.
A bare-IDENT `proof_use` source produces one lexical-use record querying only that domain; it can never resolve to a value declaration that happens to have the same spelling.
These records have no runtime declaration or value identity, but they participate in FORM-3 reservation and deterministic lexical resolution exactly at their stated scopes.
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
Complete OP-2, OP-4, OP-9, SYS-8, FN-8, FN-9, layout, address, and target-domain judgments select their own ordinary source errors.
PAR-1–PAR-3 permission and bounded-completion failures select the sequential checked lowering or an explicit unsupported target lowering, never a source rejection.
An unavailable semantic judgment or inconsistent internal derivation is a compiler failure or explicit unsupported capability, not a guessed source rejection.

A mechanical fix or restructuring is included exactly where the owning rule requires one.
Every published static diagnostic is deterministic for one compiler executable under the conditions above.
Cross-implementation byte identity is required only where this specification explicitly fixes both selection and encoding.

[DIAG-2] Successful semantic checking produces one private checked-program value bound to the exact canonical compilation unit.
It is the only input that may grant lowering authority.

The checked program explicitly represents every source operation and every compiler-derived operation required for execution, including drops, arena releases, monomorphized instances, propagation edges, every direct slice value's finite ownership-origin set, every `own slice` result's FN-1 formal return-origin ceiling and call-site substitution, and one abstract target-domain representability obligation at every runtime-sized allocation and element-address operation governed by [STOR-6].
It retains every [FN-8] GoalTemplate, its requirement occurrence `(concrete callee instance, requires_clause NodePath)`, every concrete call substitution and discharged-goal derivation, every proved body-entry requirement, and each inhabited or contradiction-proved body disposition.
It retains every proof-required integer-domain, allocation-fit, subscript-bounds, system-range, layout, address, and target-domain obligation occurrence together with the exact derivation authorizing its accepted source node.
It separately retains each successful PAR-1–PAR-3 permission and bounded-completion derivation that authorizes an optional nonsequential lowering; absence retains no permission and changes no source verdict.
It also retains every proved loop-invariant base and arbitrary-backedge judgment, each permitted exhaustion export, and every PRF-1 premise-admission, factor, scaled-sum, and final-DIRECT-residual judgment.
Target lowering must discharge each target-domain obligation from the selected target plus already-checked layout, allocation, and bounds facts before emitting the governed allocation or address operation; it may not replace a missing proof with a runtime guard.
No accepted proof-required operation carries an implicit runtime check or elimination disposition: a subscript, exact integer operation, buffer allocation, or system range is `discharged` at its owning source node, and the checked program retains its exact [ENT-4] or [ENT-6] derivation there.
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

The checked program also retains one complete source-ordered contract table, one validated conformance record per source `conform_decl`, each conformance's member-order binding vector, and every FN-4 base law derivation.
Those records are semantic evidence, not executable operations.
Ordinary lowering consumes the same checked functions and operations it would consume if those metadata tables were empty; it emits no contract or conformance object and obtains no dispatch target, check elimination, reassociation, or other optimization consequence from either a written law or a base derivation.
Any future law-driven transformation remains subject to FN-4's checked-fact and fixed transformation-applicability boundary, and to facts-off identity.

The checked-program representation is private compiler state.
Its Rust layout, allocation strategy, dense identities, instruction grouping, and internal ordering where this specification defines no semantic order are implementation-defined.
Any diagnostic or debug projection is explanatory only: it establishes no source fact and grants no lowering authority.

The language defines no writer-reachable runtime proof-failure report, because every writer-visible proof obligation is discharged before lowering or rejects the source.
Static source diagnostics follow [DIAG-1]. Diagnostic derivations follow [DIAG-2] and explain the source semantic decision; they are never reloaded or checked as a second acceptance step.
An implementation may report target qualification, unavailable external resources, trusted-computing-base failures, or compiler failures on implementation-defined channels, but none is a source-language outcome and none may be mistaken for a successful source judgment.

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
It holds in every source execution, not in a typical execution or in some execution: accepted source contains no writer-reachable proof-failure branch, and every partial operation in the window has already been discharged by its owning static Goal.
No overlapped pair reaches one state place or violates one ordinary loan except as the permission conditions above admit.
Target-resource exhaustion and trusted-computing-base termination remain outside the source execution model under [SCOPE-3] and grant no overlap permission.
No permission, submission, completion, or fast path reads a proof-failure latch or pays any other cost for a writer-reachable runtime proof fallback.
The number of workers, the identity of the host thread that executes a statement, the schedule, and whether an overlap was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Every construct of this specification defines one total sequential order over its operand evaluations, and this rule is a consumer of that order rather than a relaxation of it.
This rule uses [CAP-1]'s ordinary ownership boundary directly; it introduces no additional sharing classification.
The counted permission [PAR-2] and the staged permission [PAR-3] each form every footprint and every loan of a statement exactly as this rule forms one.

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
Every call and compiler-derived release in B has a complete target summary. Effects and ordinary loans decide interference between iterations exactly as they do between [PAR-1] calls. A `may-suspend` action requires a completion lowering that retains every iteration's argument loans until its declared milestones; an implementation without that lowering executes the loop sequentially.
Every normal continuation of every statement of B reaches L's compiler-owned binder update, so no statement of B is a `return_stmt`, a `give_stmt`, a `break_stmt` resolved to L or a loop enclosing L, or a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3].

Under a permitted overlap every state-place observable is the one produced by executing L's iterations in index order. Distinct places gain no extra cross-place order from the target mechanism [EFF-5].
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
A separate staged permission [PAR-3] cuts the body of any `for_stmt` or `loop_stmt` at its first `may-suspend` submission and admits a different overlap; it is not this permission, and neither implies the other.

[PAR-3] An implementation may execute the segments of two iterations of one `for_stmt` or `loop_stmt` body with overlapping execution only when the staged permission this rule defines holds for that loop.
This permission is distinct from [PAR-2]'s counted permission, requires none of its accumulator, combination, or index conditions, and grants a different overlap; a loop may hold either, both, or neither.
Permission holds for a loop L exactly when all of the following hold, writing B for L's body and forming every written, read, and operand-read footprint and every loan of a statement of B exactly as [PAR-1] forms one.
There is one program point c of B such that every statement of B either executes before c on every path through B or is reached only through c, and c is the argument evaluation and submission of the first `may-suspend` action of B in program order. Write P for the statements up to and including c and E for the rest.
Every edge that leaves B — a `return_stmt`, a `give_stmt` delivering outside B, a `break_stmt` resolved to L or a loop enclosing L, and a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3] — occurs in P.
An edge the statement performing c takes on the outcome of that submission, which is the edge a `let_stmt` selecting `propagate_let_rhs` at c takes, is an edge of E and not of P.
Every borrow a `may-suspend` call of B retains past its own submission is on a place rooted in a binding B itself introduces, on a place this rule replicates, or on a place no footprint of B writes. Every exclusive loan a call of E holds is on a place rooted in a binding B itself introduces or on a place this rule replicates.
Every place rooted in a binding declared outside L that a footprint of B reaches satisfies one of exactly three conditions, and a place satisfying none denies permission. Either no footprint of B writes it and every loan on it is shared; or every footprint element and every loan touching it belongs to one of P and E alone and no loan on it is retained past c; or this rule replicates it.
Every call and compiler-derived release in B has a complete target summary [FN-1, SYS-2]; a footprint element, loan, extent, or statement form the implementation does not resolve denies permission rather than granting it.
Under the staged permission an implementation may execute the segment E of one iteration with overlapping execution against the segment P of any later iteration, and against the segment E of any other iteration.
The executions of P for the iterations taken in index order do not overlap one another, and no execution of P begins before the execution of P of every earlier iteration has completed.
Every write E performs to a place rooted outside B occurs in the order of the iterations that perform it.
Every read E performs of a place rooted outside B that a footprint of B writes likewise occurs in the order of the iterations that perform it.
Under a permitted staged overlap, bindings and every Whitefoot state place equal the source-order result, on exactly the terms [PAR-2] states for its own permitted overlap.
An implementation may replicate a place, giving each concurrently executing iteration its own storage of the same length, only when that place's element type is copy [OWN-1], when no statement L's continuation reaches reads it, and when on every path through B every byte of it a footprint of B reads was written by an earlier footprint of B on that path.
The bytes one footprint reads, and the bytes it may write, are exactly those its operation contract fixes for a system operation [SYS-8], those the callee's own summary fixes for a user call after the [EFF-2] boundary projection, and the exact subscripted position for a direct element access; observing a place's length reads no byte of it.
A byte counts as written by a footprint, for the coverage condition above, only where that contract fixes that the footprint changes it: a contract stating only which bytes of a buffer may have changed [SYS-8] establishes no written byte, and a range the coverage condition needs must come from a contract that states the change exactly.
An extent the implementation does not resolve is the whole place for a read and empty for a write, and an underivable containment denies replication rather than granting it.
When an execution of one iteration leaves L through an edge of P, the overlapped execution produces exactly the observables the source-order execution produces before that point and produces none after it; every operation of an earlier iteration still outstanding is completed and its segment E performed before that edge is taken.
The host resources a system operation of L creates are not execution resources an implementation spends on overlapping. An overlapped execution delivers for each operation of L an outcome that operation could deliver in the source-order execution at that point, so an implementation whose overlap holds more such resources at once than the source-order execution holds completes the earlier iterations and performs the operation again at the source-order resource footprint before delivering any outcome.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Every partial operation in P and E has already been discharged before staged permission can be retained.
No permission, submission, completion, or fast path reads a proof-failure latch or pays any other cost for a writer-reachable runtime proof fallback.
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
fn args_count(args: &Args) -> result: own u64 reads(args);
fn arg_get(args: &Args, position: own u64) -> result: own Result<HostString, ArgError> reads(args);
fn host_bytes_len(value: &HostString) -> result: own u64 reads(value);
fn host_copy_bytes(value: &HostString, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads(value, destination), writes(destination);
fn host_utf8_len(value: &HostString) -> result: own Result<u64, Utf8Error> reads(value);
fn host_copy_utf8(value: &HostString, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads(value, destination), writes(destination);
fn relative_path(value: own HostString) -> result: own Result<RelativePath, PathError> pure;
fn open_read(permit: own FilePermit, root: &DirectoryRead, path: &RelativePath) -> result: own Result<ReadFile, IoError> reads(permit, root, path), writes(permit);
fn read_at(file: &ReadFile, destination: &uniq buffer<u8>, file_offset: own u64, start: own u64, end: own u64) -> result: own ReadOutcome reads(file, destination), writes(destination);
fn write_once(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output);
fn exit_status(code: own u8) -> result: own ExitStatus pure;
fn open_directory(permit: own FilePermit, root: &DirectoryRead, name: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<DirectoryRead, IoError> reads(permit, root, name), writes(permit);
fn open_directory_source(permit: own FilePermit, directory: &DirectoryRead) -> result: own Result<DirectorySource, IoError> reads(permit, directory), writes(permit);
fn directory_next(source: &uniq DirectorySource, destination: &uniq buffer<u8>, start: own u64, end: own u64) -> result: own ListOutcome reads(source, destination), writes(source, destination);
fn open_file(permit: own FilePermit, root: &DirectoryRead, name: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<ReadFile, IoError> reads(permit, root, name), writes(permit);
fn reserve_file(factory: &uniq FileFactory) -> result: own FilePermit reads(factory), writes(factory);
```

The inventory is therefore exactly eighteen nominal types, forty enum-variant constructors, sixty-three variant fields, sixteen operations, twenty-two operation region parameters, and forty-four operation value parameters.

Each operation's state access is fixed by its own signature. Immutable invocation and host-string state is observed through shared parameters and contributes `reads(parameter)`. Every buffer or system resource the operation changes is supplied through `&uniq` and contributes `writes(parameter)`. The lifetime on each borrow states only how long that loan lives and never appears in the row.
The rows above are exactly those state accesses; a system operation's row is declaration data and is never derived from a body, narrowed by a proof, or selected by a call site [ERR-4].
System results and written parameter components carry no proof-authority class.
A returned or loaded runtime value enters the caller only as a typed symbolic term; it establishes no proposition merely because of its producer, value, storage, or origin [SCOPE-2, ENT-3].
No system operation allocates.

Each operation additionally carries one compiler-owned target contract. This record is not source syntax and does not change contract equality.
The target contract is `never-suspends` for `args_count`, `arg_get`, `host_bytes_len`, `host_copy_bytes`, `host_utf8_len`, `host_copy_utf8`, `relative_path`, `exit_status`, and `reserve_file`. It is `may-suspend` for `open_read`, `read_at`, `write_once`, `open_directory`, `open_directory_source`, `directory_next`, and `open_file`.
A `may-suspend` operation is a finite one-shot operation. Its logical record exists before target handoff and carries separate `result-ready`, one `loan-released(path)` fact for every retained borrow, and `terminal` milestones. In this first system slice the `loan-released(path)` fact for the name an open borrows — `open_read`'s `path`, `open_file`'s and `open_directory`'s `name` — is published before target transfer, because forming the request copies the admitted code-unit range into compiler-owned storage and that copy is the operation's last access to the caller's buffer; every other applicable fact is published by the same exactly-once terminal transition. Keeping them distinct is required contract structure, not a promise that later operations publish them together.
The operation result becomes an ordinary usable source value only when its `result-ready` fact holds, and each borrow held for the call remains live until its own `loan-released(path)` fact holds. The call's ownership-complete requirement is the conjunction of its result fact and every loan-release fact the caller regains.

Every borrow keeps its ordinary [OWN-5] loan until the target contract releases it. `reserve_file` returns one fresh ordinary `FilePermit`; a successful open which returns `ReadFile`, `DirectoryRead`, or `DirectorySource` likewise produces one fresh ordinary owned value. Each result has its own binding identity and no compiler-retained parent relation to the parameter that produced it [EFF-2].

Submission has exactly three internal outcomes. `inline-terminal` publishes every promised milestone and guarantees that no later completion can arrive. `target-owned` transfers the complete operation bundle to the qualified adapter. `wait-capacity` transfers nothing to the target and retains the bundle in the runtime so another ready frame can run until bounded target capacity is available. No source value observes which outcome occurred.
A qualified target may implement this contract with native completion, readiness plus a nonblocking attempt, polling, interrupts, or a bounded blocking helper. A helper executes only the typed target adapter and publishes milestones; it never executes a writer function or writer continuation. Target completion publication may make a stackless writer frame runnable, but target code never invokes that frame directly.
No submission, completion, or target path tests a proof-failure latch or records proof-specific runtime state [EFF-4].

The only system-result propositions available to source invariants are the exact relations enumerated by [SYS-9] and the ordinary facts established by selecting one typed outcome branch.
Each such relation names its concrete call, selected outcome component, and every source actual on which the relation depends; no declaration-global fact or implicit producer class exists.
No unlisted result, projection, written parameter, field, storage value, or component establishes any relation.
Ordinary effect paths and loans describe interference independently [EFF-2, EFF-5].

Every system operation is nongeneric: it declares no type parameter and no const parameter, so no `targ` in a system-operation call is a `type` or a `const`.
A call whose callee resolves to a system operation writes its region arguments as `targs` after the `::` delimiter [GRAM-5] in declared region-parameter order and its value arguments as a `fieldinit_list` [GRAM-5] whose IDENTs equal the declared parameter names in declared order, under the same discipline [GRAM-11] applies to a user function.
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
That is the complete promise this type makes, and it is not a confinement guarantee.
An implementation presents no stronger one: a target implements directory-relative resolution with its own directory-relative facility, never by concatenating a prefix onto a path and resolving the result against an ambient working directory, and a target with no directory-relative facility fails qualification for the directory-relative semantic IDs [QUAL-1] rather than emulating them.
A confined directory state type, one guaranteeing that lexical traversal, links, mount transitions, and rename races cannot escape a granted root, is a DEFERRED addition with its own distinct contract [META-5]; a value's confinement promise is fixed by its type and never changes at runtime.
Absolute paths, cross-root operations, and target-root prefixes require their own inputs and operations, and `DirectoryRead` admits none of them [PATH-1].

[QUAL-1] Every system operation has exactly one target-independent semantic ID owned by this specification.
That ID's record binds the operation's signature, complete outcome set, ownership transitions, state effects [EFF-1], capacity behavior, completion milestones, compiler-derived cleanup [STOR-3], target action, required target guarantees [QUAL-2], and any selected-target integer-result ceiling that later target proof may consume.
The checked program carries only the semantic ID [DIAG-2]: an operation's identity comes from resolution in the system declaration domain, and no source function name or spelling, logical path [PROG-2], project, corpus, test, or signature lookalike ever selects, adds, or removes one.
A separate target-qualification table maps each `(specification version, semantic ID, target, program kind)` to exactly one approved implementation version and one private ABI symbol.
The compiler consults that table after selecting the exact target and ABI [STOR-6] and before emitting any use of the operation.
Compilation stops when the mapping is absent, when the approved implementation is incompatible with the selected target or program kind, or when a required target guarantee is unmet.
That stop is a target-qualification failure under [DIAG-1]; like a target-layout failure it is not a source-language rejection and cites no language rule.
Qualification never narrows a semantic ID to what a target can supply, and no implementation substitutes a different or weaker operation for an unqualified one.
An approved implementation may be replaced only within one semantic identity: a change to any element the record binds is a different semantic ID under a new specification version [META-5] and a compatibility review, never a target-code update.
The table is compiler-internal data; the language defines no registry, negotiation protocol, dynamic loading, or plugin interface [PROG-1].

[QUAL-2] A target qualifies for a semantic ID exactly when it supplies every target guarantee that ID's record requires; when it cannot supply one, it fails qualification for that ID and compilation stops [QUAL-1] rather than admitting the operation under a weaker guarantee.
Five guarantees are stated here because each is a property of the target with nothing in a program to check.
The first is command-lifetime argument backing: a target qualified for the command entry and for argument access supplies immutable backing for every argument code unit that is valid from before entry until the command invocation ends, either as stable native argument backing or as one complete snapshot taken before any Whitefoot code runs.
A target that can supply neither fails qualification for both IDs; a qualified target that cannot establish the backing for one invocation refuses startup before entry rather than entering with backing that does not meet this guarantee.
The second is a lossless host-string code-unit family [HOST-1] for the host-string and path semantic IDs.
The third is the target's own directory-relative resolution facility [PATH-2] for every semantic ID that resolves a relative path or one caller-supplied component against a `DirectoryRead`.
A target with no such facility fails qualification for those IDs rather than concatenating a prefix or resolving against an ambient working directory.
The fourth is a directory-enumeration facility for the enumeration semantic IDs [SYS-14]: one host call that reports a bounded batch of the entries of an open directory and advances that directory's own enumeration position.
A target with no such facility fails qualification for those IDs rather than emulating them, and in particular never substitutes a scan built out of other operations.
A fifth guarantee belongs to `host_bytes_len`: every `HostString` lease admitted by the selected implementation has a native-byte extent no greater than that target's address-index maximum, and the operation returns exactly that extent as `u64` [SYS-9].
The target stage may attach this ceiling only to the exact SSA result of the qualified semantic ID and may combine it with already-proved source bounds for layout, allocation, or address qualification [STOR-6].
It is not a target-independent source relation, does not narrow the source result type, and grants no fact to another system operation or to a value whose dataflow from this exact result is not established.
A target that supplies every required facility and guarantee but for which the table [QUAL-1] holds no approved implementation is a different stop with the same effect: compilation stops for an absent mapping, the target is not thereby declared unqualified, and no implementation is improvised for it in either case.
Qualification failure and startup refusal both occur before entry [PROG-3], so neither is a source-returned status or a recoverable outcome.

[QUAL-3] For a natively compiled command, selection is static for the whole build: [QUAL-1] fixes the approved implementation of each semantic ID at compile time, and the emitted program contains no runtime operation-ID switch, target tag, per-call dispatch table, instance handle table, or handle lookup that selects among implementations.
An `inline-terminal` transfer lowers to its required source and target checks [STOR-6], at most one direct host attempt, one count or outcome check, and a cold outcome mapper reached only on failure; it remains one completion-contract outcome rather than a second blocking source mode.
That path performs no heap allocation, no copy of the transferred data, no global system lock acquisition, and no per-call signal-disposition operation.
The compiler wrapper is inlined, or any remaining call is shown to be immaterial, as a condition of qualification.
One-time per-invocation normalization belongs to the command bootstrap before entry rather than to any transfer: on the first native command targets that bootstrap owns the process and installs the ignored disposition for the write-to-closed-pipe signal, so a closed output destination reaches source as a recoverable outcome [ERR-4].
This rule fixes the required emitted shape; the evidence establishing it is inspection of emitted code and symbols, not a machine-checked language judgment.

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
Every source-language exit follows the release table. If the host terminates execution at the external resource boundary of [SCOPE-3], Whitefoot promises neither language cleanup nor rollback of already completed system transitions.

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
The selected implementation additionally establishes [QUAL-2]'s target address-index ceiling for that returned count; this target-stage guarantee is not an ambient Whitefoot comparison fact.
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
If the host terminates the process because an external resource is unavailable under [SCOPE-3], native descriptor reclamation is host behavior rather than a Whitefoot release edge [SYS-5].

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
Static proof diagnostics use compiler output [DIAG-1]; the source runtime has no proof-reporting channel and never flushes an `Output` for a proof failure.

[SYS-13] `ExitStatus` is an opaque immutable value carrying one portable command code.
`exit_status(code)` is its one constructor: it is total and pure, every `u8` is a valid command code, so the closed code range is 0 through 255 and there is no failure outcome, no allocation, no host call, and no state effect.
`ExitStatus` is release-complete and its release is a logical consume [SYS-5].

The type is opaque rather than an alias for `u8`.
There are no implicit conversions [TYPE-4] and every value's type is exactly what its producer fixes [TYPE-5], so without a stated constructor the command entry's returned value would be unwritable; keeping the type distinct also keeps an arbitrary integer from being returned as a command status, and matches how every other system type is fixed [SYS-2].

The target maps the returned code exactly onto the host process status.
Startup or external-resource failure before entry is outside this mapping [PROG-3, SCOPE-3] and returns no Whitefoot status.

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
Each call first discharges [SYS-8]'s two static range obligations; neither operation has a runtime range check or extra effect for proof failure.
Each then validates `[start, end)` as one component before any host call: a component that is empty, longer than the target's component limit, or containing a NUL or a target separator yields `Err(InvalidPath(code: 0_u32, origin: 0_u8))`, no host call, and no resource value.
A valid range for which the directory-relative open itself fails yields the target-mapped [SYS-7] error, as `open_read` does.
After `open_file` obtains a provisional descriptor, descriptor-status inspection is required before publication: inspection failure returns its target-mapped [SYS-7] error, a successfully inspected directory returns `Err(IsDirectory(code: 0_u32, origin: 0_u8))`, and every other successfully inspected non-regular object returns `Err(Other(code: 0_u32, origin: 0_u8))`.
Before returning any of those post-open errors, `open_file` makes exactly one native close attempt, discards its close diagnostic without retry as [SYS-5] requires, and returns the inspection or synthetic classification error unchanged.
On success `open_directory` returns an independent `DirectoryRead` for the named directory and `open_file` returns an independent `ReadFile` for the named regular file; a symbolic link is not followed by either operation.

`DirectorySource` is release-complete [SYS-5].
Compiler-derived release consumes the resource and may discard only a close diagnostic, which carries no guarantee about entries already observed.
This specification declares no separate explicit-close operation, and a deep traversal therefore holds one descriptor per live level.
If the host terminates the process because an external resource is unavailable under [SCOPE-3], native descriptor reclamation is host behavior rather than a Whitefoot release edge [SYS-5].

## 18. Obligation discharge: deterministic facts, invariants, and local certificates (normative)

[ENT-1] The entailment fragment is a closed, deterministic, terminating derivation system fixed completely by this specification.
Its state is the L0 relation state, [ENT-2]'s finite signed opaque goals, [ENT-6]'s exact current-value images and specification-fixed automatic affine images, and the finite affine theorems admitted by [INV-1] and [PRF-1].
Complete-state obligation discharge [ENT-6], ordinary-call requirement discharge [FN-8], verified normal-return proof [FN-9], loop induction and program-point invariant checking [INV-1], and local certificate checking [PRF-1] are post-resolution source-acceptance judgments under [DIAG-1].
They are identical in facts-on and facts-off compilation and are not an optimizer-fact family.

The fact sources are exactly the executed control-flow edges, independently proved function requirements at callee entry, declaration and type properties fixed by this specification, constants, compiler-owned structural consequences enumerated by [ENT-3], verified earlier-SCC normal-result publications [FN-9], and machine-proved header or local invariant targets.
A runtime-origin value is an ordinary typed term in those judgments; its origin is neither a fact source nor a reason to discard an otherwise derived fact [SCOPE-2].
Only the fact sources enumerated above establish propositions; a written conclusion, unselected condition, diagnostic record, or optimizer result does not.

No source postcondition is trusted: FN-9 proves every selected exit, requires a nonempty selected-exit set, and withholds same-SCC summaries before atomic publication.
The fragment is the deterministic checker derivation of [OP-2], [OP-4], [OP-9], [FN-8], [FN-9], [INV-1], [PRF-1], [STOR-6], [SYS-8], and [DIAG-2] for the judgments this version attaches.
A solver result never participates, and no implementation may strengthen, weaken, time-bound, randomize, or truncate an unsuccessful query within the derivable set.
Every semantic candidate family and iteration count is fixed below from the complete source text; an unproved result requires exhausting its complete family regardless of elapsed time, machine speed, thread schedule, hash iteration order, or memory pressure short of [SCOPE-3]'s external resource boundary.
A successful query may retain the first witness in the specification-fixed order and omit later witnesses, because no later candidate can revoke that success; this changes diagnostic parent choice only, never the derivable set or acceptance.
An implementation may organize or cache the same derivation differently, but exceeding a wall-clock or cumulative-work budget is never a source-language verdict.
Parser, finalizer, or canonical-source invocation ceilings may stop one compiler invocation only as [DIAG-1]'s non-language `Resource` failure; they return neither source acceptance nor source rejection and may never turn an unfinished ENT-1 candidate family into `unproved`.
Two conforming implementations derive the same fact state at every applicable program point; the same FN-9 selected exits, concrete-SCC order, and established result relations; the same certificate premise, combination, and target dispositions; and the same disposition for every operation obligation, call goal, postcondition relation, and invariant.

Every nongeneric source body receives this judgment whether or not `main` reaches it.
Every generic source body additionally receives one source-schema judgment under the one source-canonical symbolic substitution formed during generic-body validation, even when it has no concrete instantiation.
That schema checks every OP-2/OP-4/OP-9/SYS-8, FN-8, and expressible FN-9 judgment its symbolic vocabulary can represent; an unproved operation is not accepted merely because no concrete instance is reachable.
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

A term is exactly one of: (a) a tracked place — a `place` [GRAM-5] whose root `pbase` IDENT resolves to any `let_stmt` binding, a `for_stmt` binder, a `param`, any match binder regardless of its [OWN-13]-derived mode, or a named const [CONST-2], formed with any number of field-selection `psuffix`es and `deref` wrappings and no subscript suffix, whose final selected type is one fragment type; (b) a length term `len(P)`, of fragment type u64, where P is a place formed under the same restriction whose final selected type is `array<T, N>`, `slice<'r, T>`, or `buffer<T>`; (c) a constant — the mathematical value of an integer literal or of an integer-typed named const, or symbolically an in-scope integer-typed const-generic parameter; (d) one of the two compiler-owned u64 capture terms belonging to an admitted `for_stmt`, identified exactly by `(that for_stmt's NodePath, lower)` or `(that for_stmt's NodePath, upper)`; (e) the one compiler-owned symbolic result datum of an admitted FN-9 clause while its RelationTemplate is formed, identified by that `ensures_clause`, its route or unrouted class, and fragment type; (f) the one compiler-owned commit value of an admitted [SET-1] `set` whose right-hand side has one fragment type, identified exactly by `(that statement's NodePath, that fragment type)`; (g) the distinguished zero term Z, used only to carry constant bounds, S7's exact mathematical-zero disequality, and [ENT-6]'s normalized integer-domain components; or (h) one compiler-owned call datum [MSR-3, ENT-3.S13], identified exactly by `(that call's NodePath, the formal ordinal, that operand's ordered projections, whether it denotes the operand's value or its length)`.
The FN-9 result datum occurs only in its template: every selected-return or caller query substitutes it with one ordinary term or constant before flow, so it never enters a body state, survives a return, or creates runtime storage.
Two places are the same term exactly when their roots resolve to the same declaration event [TYPE-6, DIAG-1] and their canonical source spellings [FORM-2] are byte-identical; a fresh binding legally reusing an expired spelling is a distinct term, and distinct spellings are distinct terms even when they resolve to overlapping storage.
Term identity thus under-approximates aliasing, while kills [ENT-5] use [OWN-7]'s resolved-place overlap relation and over-approximate it.

After TYPE-5 succeeds, each `for_stmt` endpoint atom is admitted only when its evaluated value is itself one preceding term or constant.
Any other atom is a hard error citing ENT-2 at that endpoint's `atom` node, with `SourceCoordinate` equal to its complete checked half-open source extent and the restructuring `bind the computed u64 value with one preceding ordinary let and use that term as the endpoint`.
In particular a subscripted place is not made a term by endpoint position.
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
An [ENT-6] obligation-operand form is identified by `(concrete function instance, owning obligation NodePath, operand ordinal, exact captured type, ordered projections, final result type)` and may occur only in the canonical Goal queried for that one obligation; both SystemRange conjuncts reuse the same end-operand datum.
Both forms are neither places nor L0 terms, have no direct or complete ordinary source goal origin, add no flow fact or place support, and cannot be established by naming or reevaluating their source expression.
Goal equality is exact typed tree equality, including every selected row and datum field, and therefore may hold across two source occurrences or concrete callee instances only when their complete typed trees are identical.
The finite goal universe of one concrete function is exactly the goals formed from its admitted Bool origins, requirement S4 sources, instantiated ordinary-call requirements, and the canonical OP-2, OP-9, and SYS-8 operation obligations, together with the finite parent and child trees their fixed decomposition and reconstruction rules visit.
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
Implicit facts hold at every program point: every term t carries the reflexive bound `t - t <= 0`; every term t of fragment type T carries `t - Z <= max(T)` and `Z - t <= -min(T)`; every length term over a place of type `array<T, N>` carries the equality `len(P) = N` (both bounds), with concrete N a constant and const-generic N a symbolic constant term.

[MSR-3] One denotation per operand position, keyed on the parameter's mode.
One spelling occurring at two positions of one declaration denotes two things, and which one is decided by the mode of the parameter the operand names rather than by the rule that reads it.
The complete table is:

```text
| the operand occurs in                                                | it denotes                    |
|----------------------------------------------------------------------|-------------------------------|
| a [FN-8] `requires`, naming a parameter                              | that parameter's entry image  |
| a [FN-9] `ensures`, naming an `own` or shared-borrow parameter       | that parameter's entry image  |
| a [FN-9] `ensures`, naming a `&uniq` parameter's measure             | inadmissible                  |
| a [FN-9] clause, naming the result binder                            | that result                   |
| any of the above, read at the CALLER after substitution, naming an   | that call's call datum        |
|   `own` parameter                                                    |                               |
| any of the above, read at the CALLER after substitution, naming a    | the live term                 |
|   shared-borrow parameter                                            |                               |
| any of the above, read at the CALLER after substitution, naming the  | the result                    |
|   result binder                                                      |                               |
```

An `own` operand denotes the call datum because an `own` parameter is a value the operation received and its post-state is not a thing the caller can name; and because that is what makes a relation naming a consumed operand's measure mean what it reads as, the consume the same statement performs being unable to kill a datum that contains no place.
A `&uniq` parameter's measure is inadmissible in an `ensures` because a `&uniq` parameter is the one position from which a callee could leave a caller holding a measure of a value the callee replaced: a source-declared body is a body, so a caller reading its post-state would be reading a claim about an object at a point the callee cannot name.
That inadmissibility is a hard error citing MSR-3 at the clause, with the restructuring `take the value by value and relate the result, or state the fact as a requires`.
The same operand in a `requires` stays admissible — a requirement is a fact the caller establishes before the call, not a claim about a state after it — and denotes the parameter's entry image inside the body and that call's call datum where the caller reads it, exactly as the table gives it.

A **call datum** is a compiler-owned immutable [ENT-2] term with empty support: no place occurs in it, no [ENT-5] event kills it, and no later write retargets it.
There is one former, keyed on what a datum denotes: a datum is identified by `(that call's NodePath, the formal ordinal, that operand's ordered projections, whether it denotes the operand's value or its length)`, is compiler-owned and immutable, and is established equal to that operand's pre-transfer term at the call's pre-transfer point [ENT-3.S13].
A datum is formed, never proved.
When the operand's pre-transfer term is itself immutable with empty support — a constant, a symbolic const-generic parameter, a counted capture, a commit value, or another call datum — nothing can retarget what that term denotes, so the datum is that term and no second one is formed; every other operand mints its own.
Its placement is the call, which is one of the events at which the language undertakes to carry a value's measures.
DEFERRED: the entry, construct, rebind, enum-payload-binder, and destructuring-binder placements of a measure datum, each of which carries a measured value's measures across a naming event the way the call placement carries them across a transfer; their delta is numbered rules +0 and grammar productions +0.

*Judgment:* the denotation at every operand position, and the inadmissibility of a `&uniq` parameter's measure in a source-declared `ensures`.
*Publishes:* the call datum at the call placement, and the denotation table.

[ENT-3] The fact state is defined constructively over the conservative structural normal-control graph [FN-1]: each source below establishes its L0 and signed-goal facts at its stated point; facts flow forward along normal edges; kill events apply on the edges where [ENT-5] places them, with scope-exit kills applied before any join; merge points take the [ENT-5] join and loop heads the [ENT-5] loop rule; and the state queried at any point is the [ENT-4] closure of that flow.
retired: S8
Dominated straight-line establishment is a consequence of this construction, not a second definition.
Nothing else is a fact: an `ensures_clause` is only an FN-9 proof obligation, never a trusted source; a written header or local invariant conclusion has no authority until INV-1 and any applicable PRF-1 certificate prove it; no struct invariant, compiler-invented loop proposition, inferred summary, or unverified user-function result exists.
S11 is only the compiler-owned consequence of the counted operations [FN-1] actually executes, and S12 exists only from a separately verified earlier-SCC summary under the publication formula below.
Each accepted fact retains the constructor identity and direct parents that already produced it; this diagnostic information establishes and kills no additional relation or signed goal, and no [ENT-4] answer depends on a second provenance state.

A comparison origin is defined first.
An expression has comparison origin R when (a) it is an `infix` expression whose operator is a `compare_op` — `==`, `!=`, `<`, `<=`, `>`, `>=` [OP-2] — and whose two operands are each a term or constant, R the corresponding relation over them; or (b) it is a bare IDENT naming a `let` binding of type `own Bool` whose initializer right-hand side satisfies (a) with relation R, no [ENT-5] kill event (a)–(d) applies to a fact supported by an operand term of R on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`, user-function results, and deeper indirection chains contribute no L0 comparison origin in this version; an established Boolean goal contributes relations only through the members of its signed decomposition set.

An expression has integer-domain-predicate origin G when (a) it is one total `+defined`, `-defined`, `*defined`, `/defined`, `%defined`, `ineg.defined`, `iabs.defined`, `ishl.defined`, or `ishr.defined` operation with its selected concrete operand type and complete ordered admitted value-expression identities, after every nested obligation in those operands has succeeded, G that exact typed GoalExpression; or (b) it is a bare IDENT naming an own-Bool ordinary-let binding whose initializer satisfies (a), no [ENT-5] kill event applies to G's support on any path from that initializer to the use, and the binding is the target of no `set` on any such path.
This origin is one ordinary exact goal, not a second fact channel.
Its support, expansion, kills, scope exit, joins, and signed establishment are the ordinary goal rules below.

A Bool expression has an ordinary goal origin G when, after its ordinary expression judgment and every nested operation obligation have succeeded, its completely typed expression is one admitted value expression and its root is a pure total operation-table row, with exact tree identity as [FN-8] fixes.
Construction, a user-function or system call, a move or borrow, an undischarged partial operation, an expression requiring occurrence-local evaluated-value identity, and every other expression shape has no goal origin.
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
When and only when G's complete root is one comparison admitted by comparison-origin shape (a), whose operands after template and call substitution are each an admitted term, constant, or `len(P)` length term, that exact relation R is also established.
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
`let b = buffer_new(n, v);` and `let b = buffer_vacant::<T>(n);` each establish len(b) = n on the normal continuation [OP-9], n read as term or constant.
`let m = len(P);` for a tracked P establishes m = len(P).
`let s = slice_of…(&P);` for a tracked P establishes len(s) = len(P).
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
[ENT-3.S10]
- S10 (boundary endpoint facts).
For a `match_stmt` or `value_match` whose scrutinee is directly a call to `read_at`, `write_once`, `directory_next`, `host_copy_bytes`, or `host_copy_utf8` [SYS-2, SYS-8], or a bare IDENT naming a `let` binding of that call's outcome type under the same no-kill, no-`set` path discipline as S7's checked-arithmetic origin: let s and e be the exact actuals bound to `start` and `end`, each read as a term or constant and still live at the match.
The `ReadBytes(next: w)` arm of `read_at`, the `ListBytes(next: w, entries: n)` arm of `directory_next`, and the `Ok(value: w)` arm of the other three independently establish `s <= w` and `w <= e` at arm entry; every other arm establishes neither endpoint fact.
Each result endpoint relation is derived from the operation contract with the concrete start actual as an explicit parent, so a runtime-origin start creates no unstated fact.
These facts carry the same trust class as S6's allocation-length equality — a declared operation contract, never a writer statement.
The remaining [SYS-9] relations are retained checked-program facts and are not L0 fact sources in this version.
[ENT-3.S11]
- S11 (counted-range structural facts).
In a `for_stmt` preheader, immediately after the lower and upper endpoint values have been captured once in [FN-1]'s order, establish `lower_capture = lower_endpoint` and `upper_capture = upper_endpoint`, reading each admitted endpoint as its exact term or constant, and establish `binder = lower_capture` at the compiler-owned initialization.
Close that post-capture state under [ENT-4] before [ENT-5] forms the counted head state.
On every true header edge that actually enters the body, establish `lower_capture <= binder` and `binder < upper_capture`; the first follows from initialization plus the exact representable compiler updates, and the second from the header comparison just executed.
The capture-to-endpoint equalities are established once in the preheader only: no later header or body entry rereads an endpoint or reasserts a capture equal to the current value of a mutable endpoint source.
The false header edge, every `break` edge, and the counted continuation establish no S11 fact and in particular no raw `binder = upper_capture` postcondition.
Before the binder and captures leave scope, [INV-1]'s separately proved exact-exhaustion rule may use the false guard to form a binder-free affine conclusion; that conclusion is an INV-1 fact, not S11 and not a retained binder equality.
[ENT-3.S12]
- S12 (verified user normal results).
S12 has one owning `CallResultPublication(c,q)` judgment in the ordinary semantic flow.
That judgment succeeds only when q belongs to a summary atomically published by a strictly earlier call-graph component; every actual-expression obligation and instantiated FN-8 requirement of c is discharged in the caller before transfer; every referenced formal has its exact pre-transfer substitution; ordinary consumes, borrow commits, projected effects, writes, target commits, and kills have run in the fixed order below; and every support of the substituted result relation remains live.
The candidate relation and all of those parent derivations remain private until every source-semantic judgment in the compilation unit succeeds, then enter the checked program in the same failure-atomic publication as their call and function.
Failure of any premise or any later source-semantic judgment discards the candidate and the complete prospective checked program.
This is the original construction of S12, not a second provenance pass or a check of compiler-generated data.
For one call c and verified relation q, use exactly FN-9's `A0(c)` and per-relation `M(c,q)`.
Candidate scratch establishes q once in the current ProofContext exactly as [FN-9] fixes, after ordinary transfer and every applicable consume, borrow, callee-effect, and target kill.
Each substituted formal is independent: a referenced actual that has no ENT-2 image makes only that q unavailable, while an unreferenced non-ENT-2 actual has no effect on q.
Occurrence-local call-argument evaluated-value datums never enter q.
The only destinations are the fresh direct ordinary-let binding, direct-call selected `Ok` payload, narrow direct-set receiver, and narrow selected-payload outer receiver [FN-9, ENT-5].
A named or pending outcome, stored or propagated whole outcome, false matching predicate, killed support, or rejected call establishes nothing.
The complete candidate set stays unchanged in failure-atomic scratch until the owning source judgment succeeds; any failure publishes none, and success commits all of them atomically.

[ENT-3.S13]
- S13 (call datums).
At an ordinary source call whose callee has an atomically published summary, each `own` operand of each declared relation of the resolved callee mints one call datum [MSR-3] and establishes it equal to that operand's exact pre-transfer term, at the pre-transfer point of [ENT-5]'s call-boundary order and before that boundary's consumes, borrow commits, callee-effect kills, and target kills.
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

A declared relation is **instantiated at the call**, by substituting each operand at the denotation [MSR-3]'s table gives its parameter's mode: an `own` formal by that call's call datum [ENT-3.S13], a shared-borrow formal by the live term of its resolved referent, the result binder by its destination below.
Its **support** is the ordinary L0 support of the substituted terms, taken at the call.
It is **established** on the call's normal continuation, after the call's ordinary transfer, consumes, borrow commits, target commit and kills, exactly in [ENT-5] 2898-2905's order.
A relation routed to a variant is instantiated at the call in the same order and is **restricted** to that variant's arm: it is available exactly on the paths on which that arm is entered, and it is not deferred to the arm, so an [ENT-5] event lying between the call and the arm kills a relation whose support it removes rather than preceding an establishment that has not happened.
A relation whose support is dead is not available at all; a relation over a call datum has empty support and no event kills it.
The destinations are exactly [ENT-3.S12] 2822-2837's closed list, and a relation lands nowhere else.

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

[ENT-5] The support of an L0 fact is every tracked place occurring in its terms; every compiler-owned counted capture term occurring in its terms; for each length term len(P), the root binding of P but not P's element storage — an element write never kills a length fact, because a `buffer<T>` length is fixed at allocation and an `array<T, N>` or `slice<'r, T>` length is fixed by its type or creation [TYPE-2, OP-1]; and every borrow or box/arena holder binding any of its places reads through by `deref`, a bound call-result holder included — its resolved place is the candidate actual's complete resolved place [OWN-6], so a `set` commit or projected callee write through the chain kills exactly the facts supported by that storage.
Z, literals, named const values, and call datums have empty support and never die.
A counted capture is immutable and can die only on an edge leaving its compiler-owned construct scope.

The support of either sign of an opaque goal is the union of the resolved places whose values its complete typed expression reads.
A direct binding goal therefore depends on that binding, while its separately established complete origin expansion depends on the places read by the expansion.
For a `len(P)` node, support includes P's root and every holder used to reach it but not P's element storage, under the same fixed-length boundary as an L0 length term.
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
The same overlap, holder, consume, effect, scope-exit, and counted-continuing-kill classifications below permanently invalidate it; for a `len(P)` datum the element-storage exception is the same as for ordinary length support.
A structural merge retains stability only when every reaching input retains it, and a loop head removes stability for every datum a continuing kill may invalidate.
Neither contradiction, re-establishment of a fact, assignment of an equal value, nor a later iteration restores stability.
This metadata creates no snapshot, term, relation, signed goal, or runtime action.

An L0 fact or opaque signed goal dies at the earliest of: (a) a [SET-1] `set` or [SET-2] `replace` commit whose resolved target [SET-1, SET-2, OWN-5] overlaps, under [OWN-7]'s overlap relation, the resolved place of any support member, or the compiler-owned update of a `for_stmt` binder when that binder is a support member — because a length term's support is its viewed place's non-element root path, a whole-place replace of a buffer or of any prefix of it kills that buffer's length facts, while an element-position replace, like an element write, kills none; a [SET-1] `set` commit whose right-hand side has one fragment type evaluates that right-hand side to its commit value before this kill, and after the kill exactly [ENT-3.S5]'s applicable post-write image is established; (b) a call — user function, table operation, or system operation — one of whose [EFF-2] boundary-projected `writes` occurrences projects onto a caller place or origin set containing a place that overlaps [OWN-7] the resolved place of any support member; the projection is exactly [EFF-2]'s, so a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place, and a call whose row carries no `writes` kills nothing; (c) a consuming use [OWN-1] of any support member's root; (d) an edge leaving the region of any borrow holder in its support, leaving the lexical scope of any support binding, or leaving the owning counted construct of any capture term in its support, region exit [OWN-3] included.
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
A Goal is normalized by its owning rule from that node's exact operands, types, layout, target, ownership state, effects, and completion contract.
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

The closed L0-to-affine index is formed on demand from exactly Z and each live own integer binding having both its ordinary [ENT-2] place term and a current affine value image.
For every ordered pair of those candidates whose closed L0 state has a tightest bound `left_term - right_term <= c`, substitute the candidates' current affine images to form `left_image - right_image <= c`.
For one canonical affine coefficient vector retain only the smallest upper bound; a single image whose coefficients or bound are unrepresentable in i128 is skipped and cannot suppress another image.
These retained inequalities are the `strongest canonical L0 images` below.
They are an ephemeral goal-query index over already-closed L0, not copies published into the automatic affine-premise list.

For a normalized affine inequality A, `DIRECT(A)` is exactly the following nonrecursive check, in this order: a contradictory current combined state under [ENT-4]; the strongest canonical L0 image having exactly A's canonical coefficient vector and an upper bound no greater than A's; or fixed interval substitution of every remaining atom, using its lower endpoint for a negative coefficient and upper endpoint for a positive coefficient, where each endpoint is the strongest closed L0/type bound for that atom.
`DIRECT` never selects or subtracts a published affine premise.
Every invariant conclusion and specification-fixed automatic image is appended when established to one automatic affine-premise sequence; its source category is diagnostic evidence and never partitions proof authority.
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
At each visited child, the ordinary [ENT-4] proof is tried first; when the child root is `<=`, `<`, `>=`, or `>` over values having current affine images, the checker normalizes that exact truth sign to one affine inequality and submits it to `AUTO` in the same ProofContext.
Successful children are joined only by the stated Boolean introduction node; they publish no child, parent, L0, or affine fact, and an unsuccessful candidate changes no later candidate or acceptance result.
This structural traversal invents no proposition, connective, rewrite, premise, or coefficient and is part of the single Signed Goal query rather than an [FN-8] retry or fallback.

The numeric relation domain attaches exactly four normalized families in this version.
For every source subscript `P[i]` — read, write, and [SET-1] target position alike — SubscriptBounds is `i < len(P)`, normalized `i - len(P) <= -1`, at that subscript's `psuffix` node.
There is one obligation per subscript in a chain.
The offset has exact type `own u64` [OP-4], so the relation is over the two u64 mathematical values.
A subscript has no separate opaque signed-goal identity for its own bounds obligation; after that obligation succeeds, its selected structural index row may occur as a value child of another exact Goal as [ENT-2] fixes.
Its fixed BoundedRelation checker first accepts contradiction or the exact closed L0 bound and refutes a closed L0 negation.
For an `array<T, N>` whose selected N is a concrete value in this instance, if the offset has a current affine image, it next submits `i <= N - 1` to `AUTO` and composes that result with the implicit L0 equality `N = len(P)`.
It then visits every live own integer binding m having a current affine image, in compiler-owned source-allocation order; when closed L0 has its tightest `m - len(P) <= c`, it submits the one exact target `image(i) - image(m) <= -1 - c` to `AUTO` and composes a success transitively with that L0 bridge.
An unavailable image or unrepresentable candidate is skipped without suppressing a later candidate; otherwise the obligation is unproved.
A refuted or unproved occurrence is an OP-4 rejection and publishes no checked program.

IntegerDomain attaches one obligation to every proof-required exact integer occurrence [OP-2] at its `infix` or `call` node.
Its canonical goal is always the corresponding total `.defined` operation with the same selected concrete type and complete ordered operand identities.
Each operand uses its stable value expression when available after all nested obligations have succeeded, otherwise that obligation's occurrence-local evaluated-value datum [ENT-2].
The fixed disposition order is: contradiction; exact positive goal; exact negative goal; the finite L0 normalization below proving true or false; the fixed affine normalization below; the fixed nonconstant-product rule below; otherwise unproved.
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
After contradiction, an exact positive Goal discharges and an exact negative Goal refutes; otherwise its normalization is `n <= floor((2^64 - 1) / stride_ceiling(S))` for the selected stored type S.
The closed L0 relation is tried when available, followed by `AUTO` over the same normalized relation when n has a current affine image; a derived false comparison refutes, and absence of every proof is unproved.
A refuted or unproved occurrence is an OP-9 rejection and creates no allocation or runtime operation.

SystemRange attaches two independent Goals in declared order to each [SYS-8] range-bearing call: ordinal zero is `start <= end`; ordinal one is `end <= len(buffer)`.
Each value child uses the same stable-or-occurrence-local identity rule; the one evaluated end identity is constructed once and shared by both Goals, and every SystemRange ordinal therefore has a canonical Goal even when no L0 normalization exists.
Each uses the same BoundedRelation checker as SubscriptBounds, additionally carrying its exact signed comparison Goal and its direct affine normalization when formable.
It therefore checks contradiction, the exact positive or negative signed fact, closed L0, direct `AUTO`, and the fixed affine-left/L0-right bridge in that order; a result from one ordinal supplies no premise to the other.
The first refuted or unproved Goal is a SYS-8 rejection and creates no host call, runtime condition, effect, or trap.

Initialization, ownership and loans, state effects, layout and address formation, selected-target integer domains, and bounded parallel queue/completion protocols keep their own finite proposition and checker domains under their owning numbered rules.
They use the same fail-closed Goal/checker principle; a later domain needing a source numeric conclusion consumes the checked conclusion retained from ProofContext rather than repeating its derivation. They are not encoded as numeric L0 relations merely to make one universal solver, and they do not become a second authority for accepting source propositions.
Each checker has a specification-fixed finite algorithm whose complete work is a deterministic function of its source-derived input, a unique closure or result, a deterministic diagnostic order, and no timeout-selected acceptance.

The mechanical repairs for an unproved Goal are a dominating source branch whose false edge handles the domain outcome, a preceding proved invariant whose optional [PRF-1] block names sufficient premises, or a verified callee relation [FN-9].
For a subscripted offset that is not itself an [ENT-2] term, first bind the inner read with one ordinary `let` and, where required, one total `cvt`; its own inner obligation is discharged independently.
Writing a proposition without one of these derivations establishes nothing.

Each concrete obligation identity is `(concrete function instance, exact source NodePath, family ordinal)`.
SubscriptBounds, IntegerDomain, and AllocationFit use ordinal zero; SystemRange uses zero for `start <= end` and one for `end <= len(buffer)`.
A requirement occurrence is `(concrete function instance, requires_clause NodePath)` [DIAG-2].
These identities do not participate in Goal equality [FN-8].
The checked program retains the accepted Goal, its deterministic derivation root, and its erased disposition for diagnostics and proof consumers.
Internal derivation metadata is only the diagnostic explanation of that acceptance decision and establishes nothing independently.
[INV-1] A `header_invariant` and an `invariant_stmt` are two placements of the same proof-only declaration: the writer states that one ordered affine relation holds at that exact source point, and the checker must prove it before the relation gains authority.
A loop-header placement additionally creates induction obligations because control may enter that point from the preheader and from a backedge; a body placement creates only the one ordinary program-point obligation in its entering ProofContext.
The spelling `invariant` therefore describes the writer-visible meaning in both positions, while the control-flow owner determines how many incoming-edge obligations exist.

The `compare_op` of a `header_invariant`, an `invariant_stmt`, or a relation-form `proof_use` must be exactly `<=`, `<`, `>=`, or `>`; it selects a proof-domain relation over its two affine expressions and performs no [OP-1] operation, and `==` or `!=` in that position is a hard error citing INV-1 at the `compare_op` node.
The checker normalizes `a <= b` to `a-b <= 0`, `a < b` to `a-b <= -1`, `a >= b` to `b-a <= 0`, and `a > b` to `b-a <= -1`.
Equality, disequality, and every other Bool root are outside this version's invariant surface.

An `affine_expr` denotes a mathematical integer expression and never a runtime evaluation.
At a counted-loop header an IDENT may resolve to that header's `for_binding` binder or to a live own-mode integer value in the preheader.
At an ordinary-loop header it may resolve only to a live own-mode integer value in the preheader.
At an `invariant_stmt` it may resolve only to a live own-mode integer value in the statement's entering lexical context.
An integer-typed named const is admitted and denotes the one closed value it declares, folded to that value at formation; it is already an [ENT-2] constant term, so it means in a relation exactly what it means everywhere else. An integer-typed const-generic parameter is symbolic rather than closed and is not this admission. Calls, construction, dereference, subscript, field selection, allocation, borrow holders, moved values, and every other runtime expression form are not admitted as affine atoms in this version.
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

For the base batch, the checker proves every header target with `AUTO` in the complete preheader state and assumes no conclusion from that same header.
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
Without a proof block its target must succeed under `AUTO`.
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

A bare-IDENT source in `proof_use` resolves in the invariant-name domain to the exact immutable normalized target published by that dominating invariant declaration; it is not reparsed using the current value bound to each source spelling.
The checked reference stores `(concrete function instance, invariant declaration identity)` obtained from that lexical resolution, never the IDENT spelling or a later spelling lookup.
Inside a loop body a header declaration identity denotes the currently activated arbitrary-iteration header theorem, not its base value images and not the still-unproved next-header target.
A relation-form source in `proof_use` uses INV-1's exact affine formation and normalization rules, including substitution of every referenced local's current value image before canonical normalization. It is owned diagnostically by PRF-1 and must itself be proved by `AUTO`.
A named source must be in lexical scope and its published fact must be available in the entering ProofContext.
Every use, named or written, is checked against the same snapshot immediately before the owning `invariant_stmt`.
No use publishes a fact, and no earlier use can help prove a later use.

The optional bare-decimal multiplier in `proof_use` is a proof-domain positive integer factor.
A multiplied relation-form source writes its relation in parentheses, `use 3 * (a <= b);`, an unmultiplied relation-form source is bare, `use a <= b;`, and a named source is never parenthesized [GRAM-4]; those parentheses delimit the premise the factor scales and are the grammar's own, not an affine grouping.
Its canonical spelling is one bare decimal with no leading zero, its value must be in `2..=i128::MAX`, and omission alone means one.
It is neither a source integer literal nor a runtime type.
Zero, an explicit one, a leading zero, an out-of-range factor, a negative or typed literal, and arithmetic overflow reject.
No two `proof_use` entries may resolve to the same normalized premise, regardless of factor; their total scaling must be expressed by one factor on one use.
No global or subset-minimality judgment is performed on the remaining list.

The checker forms S independently of premise admission by multiplying each normalized premise by its written positive factor and adding the results in source order with checked `i128` arithmetic; acceptance additionally requires every source to be independently admitted.
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
A resolved but unavailable named source, undischarged or malformed relation source, invalid factor, duplicate source, arithmetic or structural overflow, failed final `DIRECT` residual, or redundant block cites PRF-1 at the smallest owning source node and publishes no target.

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
  if x < 0_i32 {
    return Neg();
  } else if x == 0_i32 {
    return Zero();
  } else {
    return Pos();
  }
}

command fn main() -> status: own ExitStatus pure {
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

## 20. Spec meta-rules (CI-checked)

[META-1] Spec-CI enforces the regularity invariants defined elsewhere: one spelling per construct [FORM-1] and a 1:1 production-to-core-tree-node mapping [GRAM-1].
Its unique machine-checked content is that no rule ID is defined twice and every cross-reference resolves [META-4, META-6].
[META-2] No context-dependent spellings or rule variants: no rule's meaning depends on surrounding context; defaulting rules do not exist.
[META-3] No rule carries an exception clause; conditional structure is expressed as total positive rules or table data.
[META-4] Every normative fact is stated once; other mentions are rule-ID cross-references.
[META-5] Every change to this artifact declares its spec delta (rules ±, tokens ±, spellings ±, exceptions ±) and its SELECTION GROUND (evidence-selected vs minimality-selected) in this document's status header.
`docs/WORKFLOW.md` defines the repository's four branch-and-main rules: work-branch changes need no approval, while merging into `main` requires owner approval of the exact tested revision and the records those rules require.
DEFERRED markers are tracked specification-delta obligations and do not create another approval point.
[META-6] Every rule carries an entry in `spec/derivation/derivation-ledger.md` tracing it to `docs/constitution.md`; a rule whose chain is refuted or orphaned (evidence card dies, constitutional premise amended) is flagged for re-grounding, and underived rules may not ratify.
The native `whitefoot-spec` gate checks that every active rule ID has a ledger row.
