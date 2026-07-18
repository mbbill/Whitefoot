# Batch-1 spec-fix deltas (proposals, NOT yet applied)

Source: workflow `wf_2158a9c2-223` (6 clusters x propose->critique->synthesize, 36 agents). Angle brackets un-escaped. These are PROPOSALS awaiting owner sign-off; the canonical spec is untouched.



## Cluster: tier0-errata — Tier-0 internal defects (self-contradictions)

- confidence: **high** · selection_ground: mixed · form1_breaking: True · needs_experiment: True
- changed/new rule IDs: FORM-2, FORM-3, FORM-5, FORM-7, GRAM-2, GRAM-5, TYPE-4, TYPE-7, OP-1, OP-2, OP-6, CONST-1, PRE-1, META-1, EX-1

### Recommendation
Merge, not a single winner. Skeleton from Proposal 2 (first-principles mutual-derivability: the canonical grammar must accept EX-1, the FORM-2 formatter must re-emit it, the checker must accept it, every op must be total/UB-free). Take Proposals 1/2's exact-or-Result cvt (29-pair value-preserving partition, arithmetic-verified) and REJECT Proposal 3's "Rounding-total" cvt category outright — both critics showed it re-opens D5 by making int→float / f64→f32 total with SILENT round-ties-to-even (a W3/R4 silent-corruption channel that also erases cvt's own claimed Rust delta). Take Proposal 2's COMPLETE FORM-2 (both critics verified colon-only under-fixes: EX-1 needs `(` `<` in no-space-before and `&` in no-space-after, else `sign_of (x)`, `iadd.checked <i32>`, `& 'r a`). Take Proposal 2's explicit deref typing (TYPE-7) + reserved-op-names; Proposal 1's simpler NFC-free string; Proposal 3's D10 red-CI catch and NaN-special-cased float→float exactness lowering (drop Proposal 1's saturate-roundtrip test, which wrongly returns Ok at INT_MAX+1).

The load-bearing NEW work beyond all three proposals: I close the DEEPER D1 root both critics flagged and none fixed. FORM-3's OPNAME `[a-z][a-z0-9_]*\.[a-z]+` COLLIDES with field-access `psuffix := "." IDENT` under FORM-3 maximal munch — I verified `p.x`, `foo.bar`, `result.field` each munch as one OPNAME token and `p.bar2` mangles to `p.bar`, so `callee := IDENT | OPNAME` alone is only sound because EX-1 has no field access. Fix: tighten OPNAME to a CLOSED mode-suffix set `\.(wrap|trap|checked|strict)` so a dotted place can never munch as an op name (verified: `p.x`/`foo.bar`/`p.bar2` now lex as IDENT `.` IDENT), plus reserve the four mode-words as identifiers to kill the residual (`s.checked` still munches — verified). This is byte-stable for EX-1 and every current op. I keep the dotted spelling as the LEADING candidate because it costs zero canonical-byte churn; the regularity-maximal alternative (flatten every op to underscore `iadd_checked`, delete the OPNAME class, make `.` mean field-access only, no reserved words, `call := IDENT` needs no edit) is a live owner ruling with a W1 dimension.

Why this beats the alternatives after critique: it is the only package that (a) makes EX-1 fully mutually-derivable (parse + FORM-2 reprint + ownership-accept), (b) reintroduces NO silent-value channel (cvt exact-or-Result, no rounding op anywhere), (c) closes the field-access collision the winning proposals missed, and (d) is honest about what the mechanical guard can catch (D1/D2 now; D3 only once a type layer exists — checker.py is ownership-only and today ACCEPTS the buggy borrow-as-value). cvt and DivError are reframed as W3/T2 wins, not P0 wins, per both critics: the total cvt pairs lower to sext/zext/fpext (bit-identical to Rust `as`, zero P0 delta) and the Ok-path divisor!=0 fact comes from the guard, not the enum's variant count.

### Spec changes (apply-ready)
All edits are OLD -> NEW against kernel-spec-v0.4.md. New rule text is phrased as total positive rules and avoids the tokens "unless"/"except that" so tools/spec_ci.py's META-3 smell scan stays green.

=== D10 PREREQUISITE (unblock red CI before any guard) — spec/derivation-ledger.md line 90 ===
OLD: `| DIAG-3(v0.4.1: schemas delivered) | Report family: check, lifetime, trap, check-density reports | 🟡 existence-only | ...`
NEW: `| DIAG-3 | Report family: check, lifetime, trap, check-density reports (v0.4.1: schemas delivered) | 🟡 existence-only | ...`
(Only the row KEY changes: spec_ci probes the literal `| DIAG-3 |`; the annotation moves into the claim cell. Verified: this is the sole current META-6 violation, exit 1 -> 0.)

=== D1 (call production + OPNAME/field-access collision) ===
FORM-3 (line 25) OLD: `OPNAME `[a-z][a-z0-9_]*\.[a-z]+` (single token, e.g. `iadd.wrap`).`
NEW: `OPNAME `[a-z][a-z0-9_]*\.(wrap|trap|checked|strict)` (single token; base is an IDENT and the mode suffix is a CLOSED word set, so an OPNAME can never maximal-munch a field-access place `p.field` [GRAM-5], e.g. `iadd.checked`).`

GRAM-5 (lines 100-101) OLD:
`expr      := literal | "move" place | place | call | construct | borrow_expr`
`call      := IDENT targs? "(" arg_list? ")"`
NEW:
`expr      := literal | "move" place | place | call | construct | borrow_expr`
`call      := callee targs? "(" arg_list? ")"`
`callee    := IDENT | OPNAME`
(GRAM-1 determinism preserved: OPNAME and IDENT are lexically disjoint at the FORM-3 dot-decision, and call-vs-place still resolves on the second token — IDENT-then-`.` is a field place, IDENT-then-`(`/`<` and OPNAME are calls.)

OP-1 (after line 194 table) APPEND clause:
`An operation name is an OPNAME (dotted, closed mode-suffix, e.g. `iadd.checked`) or a dotless IDENT (`ieq ine ilt ile igt ige feq flt fle band bor bxor bnot cvt len slice_of box_new arena_new`); both are consumed by `call` [GRAM-5] and resolved by NAME LOOKUP — an OPNAME callee names its table op; an IDENT callee names its table op when this table defines that spelling, otherwise a program `fn_decl` of that spelling; a callee in neither is a hard error [DIAG-1]. The dotless operation IDENTs above and the mode-words `wrap` `trap` `checked` `strict` are RESERVED: no `fn_decl`, field, param, binder, or region binds them (this keeps op-vs-fn resolution context-free [META-2] and keeps a field-access place `p.field` from lexing as an OPNAME [FORM-3]).`

=== D2 (FORM-2 byte format not total; EX-1 not formatter-derivable) ===
FORM-2 (line 23) OLD fragment: `no space after `(` `<` or before `)` `>` `,` `;` `.``
NEW fragment: `no space after `(` `<` `&` or before `)` `>` `,` `;` `.` `:` `(` `<``
(Verified rendering all of EX-1 exactly: `sign_of(x)`, `iadd.checked<i32>(deref(p), 2_i32)`, `a: own i32`, `&'r a`, `Result<i32, Overflow>`. `&` glues shared-borrow `&'r`; `&uniq` ends alpha so keeps its default space in `&uniq 'r`.)

=== D3 (EX-1 passes a borrow where a value is wanted) ===
Add new rule after TYPE-6 (before CONST-1):
`[TYPE-7] Reading through a reference is explicit. `deref(place)` where place has type `&'r T`, `&uniq 'r T`, `box<T>`, or `arena<'r, T>` denotes a place of referent type T [GRAM-5]; a use of that place copies it when T is copy and requires `move` when T is affine [OWN-1]. A borrow-mode or box/arena binding used where a value of its referent type T is expected is a type error [TYPE-5], with the mechanical fix `deref(·)`. There is no implicit read-through-borrow [TYPE-4, META-2].`
Re-cut EX-1 (line 320) OLD: `    let s: own Result<i32, Overflow> = iadd.checked<i32>(p, 2_i32);`
NEW: `    let s: own Result<i32, Overflow> = iadd.checked<i32>(deref(p), 2_i32);`
(Reject the auto read-through-borrow anti-rule: it is exactly the implicit &i32->i32 conversion TYPE-4 forbids and the context-dependent read META-2 forbids. Verified checker.py accepts `deref(p)` via resolve() through the borrow holder.)

=== D4 (out-of-range / non-canonical numeric literals) ===
Add new rule after FORM-6:
`[FORM-7] Numeric-literal well-formedness (R4 check-reject). An integer literal `d_T` is legal iff its value d lies in the closed range of T — `[0, 2^K − 1]` for `uK`, `[0, 2^(K−1) − 1]` for `iK` — and has no leading zeros (the single digit `0` is its own canonical form). A float literal `m_T` is legal iff its round-to-nearest-even value in T is finite. An out-of-range integer, a leading-zero integer, or a float that rounds to ±inf is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or undefined value. There are no negative literals — negative values are produced by `ineg.*` [OP-1]. The canonical decimal spelling of a float value is gated on the FORM-1 reject-vs-canonicalize decision and DEFERRED [META-5].`

=== D5 + D6 (cvt totality contradiction + undefined partition) ===
TYPE-4 (line 124) OLD: `[TYPE-4] There are no implicit conversions. Representation changes are explicit ops: `cvt<Src, Dst>(x)` is total where value-preserving for all inputs, and returns `Result<Dst, NarrowError>` otherwise (per the operation table, §7).`
NEW: `[TYPE-4] There are no implicit conversions. Representation change is the single explicit op `cvt<Src, Dst>(x)`. Totality is decided by value-preservation, not bit-width: `cvt` returns `own Dst` (total) iff every value of Src is EXACTLY representable in Dst, and `own Result<Dst, NarrowError>` otherwise; it never rounds, truncates, or saturates. The exact partition and per-value semantics are [OP-6]. Deliberate rounding is a separate DEFERRED float-round op family, never `cvt`.`
OP-1 (lines 189-190) OLD:
`| `cvt` | widening int/float pairs | `(Src) -> own Dst` | pure |`
`| `cvt` | narrowing pairs | `(Src) -> own Result<Dst, NarrowError>` | pure |`
NEW:
`| `cvt` | value-preserving pairs [OP-6] | `(Src) -> own Dst` | pure |`
`| `cvt` | all other distinct numeric pairs [OP-6] | `(Src) -> own Result<Dst, NarrowError>` | pure |`
Add new rule after OP-5:
`[OP-6] cvt partition and semantics (cross-ref TYPE-4). `cvt<Src, Dst>` is defined for every ordered pair of distinct numeric primitives; `cvt<T, T>` is not an operation. cvt is EXACT: it yields `Ok(y)` when the Src value x is exactly representable in Dst (y the unique such Dst value), and `Err(NarrowError())` otherwise; cvt never rounds, truncates, or saturates. A non-integral float->int, an out-of-range value, a value not exactly representable in a narrower float, and any NaN or ±inf targeting an integer all yield `Err`. For float->float, ±inf maps to ±inf and NaN maps to the target canonical quiet-NaN, both value-preserving. A pair is TOTAL — signature `(Src) -> own Dst`, no Result — iff every Src value is exactly representable in Dst; the total pairs are exactly these 29: `iN->iM` and `uN->uM` for N<M; `uN->iM` for N<M; `{i8,i16,u8,u16}->f32`; `{i8,i16,i32,u8,u16,u32}->f64`; `f32->f64`. Every other distinct numeric pair returns `(Src) -> own Result<Dst, NarrowError>`.`
(Lowering, all confirmed compiling on local clang 21: total pairs -> `sext`/`zext`/`fpext`, no check. Result float->int -> explicit guards `fcmp uno` (NaN), range compares vs Dst bounds, and `fcmp one x, @llvm.trunc.fM(x)` (non-integral); on pass, `@llvm.fptosi.sat.iN.fM`/`@llvm.fptoui.sat.iN.fM` on the proven in-range integral value — never raw `fptosi` (UB). Result int->float and f64->f32 -> cast then cast-back `fcmp oeq`, with NaN special-cased to Ok (feq(NaN,NaN)=false would otherwise wrongly Err). Do NOT use a saturate-then-roundtrip-only test — it wrongly returns Ok at INT_MAX+1.)

=== D7 (idiv/irem second failure INT_MIN/-1; trap behavior) ===
PRE-1 (line 284) OLD: `enum DivideByZero { DivideByZero(); }`
NEW: `enum DivError { DivideByZero(); DivOverflow(); }`
OP-1 (line 180) OLD: `| `idiv.checked` `irem.checked` | all int T | `(T, T) -> own Result<T, DivideByZero>` | pure |`
NEW: `| `idiv.checked` `irem.checked` | all int T | `(T, T) -> own Result<T, DivError>` | pure |`
OP-2 (line 196) APPEND to the rule: ` Integer division and remainder have two checkable failures: a zero divisor (all int T) and, for signed T only, the single signed-overflow case `INT_MIN / −1` (LLVM `sdiv`/`srem` are UB on both). `.trap` traps on either; `.checked` returns `Err(DivideByZero())` for a zero divisor and `Err(DivOverflow())` for signed `INT_MIN / −1`, else `Ok`. Unsigned T cannot overflow, so `DivOverflow` is statically unreachable for unsigned T; the uniform `DivError` type is retained for regularity. Both failures are table-fixed classifications [ERR-4], never call-site-dependent.`
(Lowering confirmed: guard `icmp eq %d, 0` (all int) and, signed, `and(icmp eq %n, INT_MIN, icmp eq %d, -1)` before `sdiv`/`srem`; unsigned uses only the zero guard before `udiv`/`urem`. Removing the standalone `DivideByZero` enum keeps every variant name globally unique, so democ's vtag resolves with no collision.)

=== D8 (dead const-param gparam) ===
GRAM-2 (line 57) OLD: `gparam       := TYPEID (":" TYPEID)? | "const" IDENT ":" type`
NEW: `gparam       := TYPEID (":" TYPEID)?`
CONST-1 (line 130) APPEND: ` Const-generic PARAMETERS are also DEFERRED (recorded delta: −1 gparam alternative removed now [GRAM-2]; +const-param wiring through `const` and `targ` positions when the sublanguage lands). v0 has no const-generic capability; `array<T, N>` uses a literal N only [GRAM-3].`

=== D9 (META-4 self-violation: FORM-1 and META-1 both state one-spelling) ===
META-1 (line 336) OLD: `[META-1] One spelling per construct; productions map 1:1 to core-tree nodes.`
NEW: `[META-1] Spec-CI enforces the regularity invariants defined elsewhere — one spelling per construct [FORM-1] and 1:1 production-to-core-tree-node mapping [GRAM-1]. Its unique machine-checked content: no rule ID is defined twice and every cross-reference resolves [META-4, META-6].`

=== D11 (STRING interior underspecified) ===
FORM-5 (line 29) OLD fragment: `STRING `"..."` with escapes `\\ \" \n` only, one canonical escape per character (a string value has one spelling). STRING appears only in `doc` and `check` messages.`
NEW fragment: `STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in `U+0020..U+007E` other than `"` (U+0022) and `\` (U+005C), or one of exactly three escapes `\\` `\"` `\n`; no other byte is legal. Each character has exactly one legal spelling — the escape where one is defined, the raw byte otherwise. STRING appears only in `doc` and `check` messages [GRAM-3]. (Non-ASCII diagnostic text is DEFERRED; UTF-8/NFC strings are an owner ruling if ever needed.)`
(Adopts the implementation critic's minimal missing option: STRING is diagnostic-only and IDENT/TYPEID are already ASCII, so ASCII-printable eliminates the entire NFC/UCD/Unicode-version-pinning burden and its canonical-byte drift hazard — strictly simpler than Proposals 2/3's property gauntlet.)

=== RECOMMENDED spec_ci GUARD (mechanical D1/D2 catch; D3 pending type layer) ===
After the D10 key fix, add tools/spec_ci.py checks: (1) extract the EX-1 fenced block; (2) lex with a FORM-3 lexer (closed-suffix OPNAME, no `.`-special-case) and parse with a GRAM-2..7 parser — an unconsumable token FAILs (catches D1); (3) FORM-2 canonical-reprint the tree and assert byte-equality (catches D2 and any FORM-2 drift); (4) run checker.py ownership pass and assert ACCEPT. Add a lexer unit assertion that `s.field` lexes as three tokens (guards the D1 collision fix). Single-source ONE machine-readable grammar+op-table consumed by BOTH spec_ci and democ (democ's `.`-special-case + `[a-z][a-z0-9_]*(?:\.[a-z]+)?` tokenizer is the divergence that hid D1; retune its OPNAME to `\.(?:wrap|trap|checked|strict)`). HONEST SCOPE: D3 (borrow-as-value) is invisible until a typed arity pass exists — checker.py is ownership-only and today accepts `iadd.checked(p, ...)`; state the guard catches D1/D2 now, D3 after the type layer.

### Draft derivation-ledger rows
Three NEW rows (required or spec_ci META-6 goes red), plus a re-grounding note for changed rows:

| OP-6 | cvt exact-or-Result partition (29 value-preserving total pairs; no rounding) | ✅ derived | TYPE-4 (no implicit value-changing conversion, round-2 decided law) + W3 (no silent value change — the named R0 delta over Rust `as`) + R4 (non-exact -> checked Result, never silent) + T2 (float->int via fptosi.sat + guards, never raw fptosi UB). 29-pair total set arithmetic-verified against 2^24/2^53 exact-integer bounds. | EVIDENCE-selected: partition forced by exact-representability. Provisional sub-part: "cvt does no rounding" leaves fractional/lossy conversion to a deferred float-round op (minimality-consistent with v0 float scope). |

| TYPE-7 | deref typing; no implicit read-through-borrow | ✅ derived | TYPE-4 (no implicit conversions) + META-2 (no context-dependent meaning) + OWN-1 (copy-on-use for primitives) + OWN-5 (reads through shared borrow permitted). Explicit deref is the only regular reconciliation; the auto read-through-borrow alternative violates TYPE-4 + META-2. Fills a real typing gap (deref had no typing rule). | EVIDENCE-selected. |

| FORM-7 | numeric-literal range/leading-zero/non-finite reject | ✅ derived | T2 (no undefined value) + R4 (check-reject dominates silent-corruption) + W3 (an out-of-range literal is the silent-corruption channel a cheat-proof language forbids) + FORM-1 (one canonical spelling -> integer leading-zero reject). | MIXED: range + non-finite reject are R4-forced (evidence-selected); the float decimal canonical form is DEFERRED to the FORM-1 reject-vs-canonicalize gate (minimality-selected, provisional). |

Re-grounding (META-6 auto-flags these changed rows; update their chains to cite the new sub-parts): FORM-2 (+3 no-space members, EX-1-derivability), FORM-3 (closed-suffix OPNAME), FORM-5 (ASCII STRING), GRAM-2 (−1 gparam alt), GRAM-5 (callee production), TYPE-4 (value-preservation replaces widening), OP-1 (+cvt rows, +DivError, +reserved-names clause), OP-2 (+two div failures), CONST-1 (+const-param deferral), PRE-1 (DivError replaces DivideByZero), META-1 (cross-ref rewrite), EX-1 (deref re-cut + FORM-2 byte-exact).

### Rust (R0) delta
Two substantive deltas; rest hygiene/parity, named honestly. (1) cvt exact-or-Result vs Rust `as`: Rust `as` silently truncates ints, rounds float->int, saturates (since 1.45); TryFrom never covers float->int exactness. Whitefoot cvt NEVER changes a value silently. Reframed per both critics as a W3/T2 cheat-proofness win, NOT P0 — total pairs lower to sext/zext/fpext (bit-identical to `as`, zero P0 delta), and the "narrowed result fits Dst" range fact is realized only if the checked-Ok path emits !range/llvm.assume (recommended, absent today). (2) idiv/irem `.checked` returns distinguished DivError{DivideByZero; DivOverflow} vs Rust checked_div folding both into None — a W3/ERR-2 win, not P0 (the Ok-path divisor!=0 fact comes from the guard, not variant count). The genuine R0 delta is `.trap`: traps on INT_MIN/-1 and zero-divisor in ALL builds (one semantics), vs Rust panic-debug/wrap-release. Parity/floors: D4 reject matches Rust E0080; D11 ASCII strings; D1/D2/D8/D9 internal hygiene. Honest negative delta: D8 leaves v0 with NO const generics (Rust has them). LEX-1: deref = explicit LLVM-load semantics, no Rust auto-deref/Deref-coercion (the context-dependent read META-2 forbids).

### Experiment spec
Two A/Bs, both requiring the not-yet-built M3 AI-codegen harness. PRIMARY (D7 error typing): Arm A = uniform `enum DivError { DivideByZero(); DivOverflow(); }` for all int T (max regularity, one prelude form, global variant-name uniqueness; cost: a writer destructuring an UNSIGNED div/rem error must fill a statically-dead `DivOverflow` arm per ERR-2 no-wildcard). Arm B = signedness-split return type (signed T -> DivError, unsigned T -> a divide-by-zero-only type; no dead arm, but the return type varies by type-arg signedness, cvt-precedented, and needs either a second divide-by-zero spelling or type-directed variant resolution). Corpus: unsigned-division tasks that DESTRUCTURE the div error (not just try-propagate). Metrics: first-attempt compile-correct rate and repair-loops-to-green. Models: the target LOW-capability writer tier (the W1 subject) + one mid tier as control. Leading candidate pending result: Arm A (retained above). SECONDARY (D1 op-name spelling): Arm A = closed-suffix dotted `iadd.checked` (byte-stable, reserves 4 mode-words) vs Arm B = flatten-to-underscore `iadd_checked` (deletes OPNAME class, `.` means field-access only, no reserved words, `call := IDENT` unchanged). Corpus: tasks emitting mode-suffixed arithmetic from the in-context spec. Metric: low-cap writer op-spelling error rate. Leading candidate: Arm A (zero canonical-byte churn); flatten wins only if it measurably lowers writer error.

### Remaining owner rulings (6)
1. INT_MIN inexpressibility: with no negative literals and FORM-7's signed range [0, 2^(K−1)−1], iK::MIN (= −2^(K−1)) cannot be written to then negate. Ruling: prelude MIN/MAX consts, a signed-MIN literal exception, or defer to the const sublanguage. Must resolve before signed-MIN is needed.
2. Bare-TYPEID variant resolution (Tier-0-adjacent): the spec is SILENT on how DivideByZero()/DivOverflow()/Ok()/etc. resolve (democ assumes global variant-name uniqueness; TYPE-5 full annotations could permit expected-type-directed resolution). My D7 fix keeps names globally unique so no collision now, but the resolution rule must be pinned before user enums proliferate.
3. D1 op-name spelling: closed-suffix-dotted (leading, byte-stable, reserves wrap/trap/checked/strict) vs flatten-to-underscore (regularity-maximal, deletes OPNAME class). Aesthetic + the W1 experiment above.
4. Float literal canonical decimal form (FORM-7 deferral): follows the global FORM-1 reject-vs-canonicalize (P12) decision — reject non-canonical vs gofmt-style canonicalize.
5. Non-ASCII diagnostic strings (D11): confirm ASCII-only STRING is acceptable for doc/check messages, or card UTF-8/NFC with a pinned Unicode version if non-ASCII is ever wanted.
6. cvt P0 materialization: decide whether the checked-Ok narrowing path emits !range/llvm.assume so a range-analysis pass consumes the 'fits Dst' fact — required if cvt is to carry any P0 (not just W3) delta over Rust `as`.


## Cluster: optable-completion — Operation-table + literal completion (bit + numeric/float)

- confidence: **high** · selection_ground: mixed · form1_breaking: True · needs_experiment: True
- changed/new rule IDs: OP-1, OP-2, OP-3, OP-6, OP-7, FORM-5, PRE-1

### Recommendation
Merge P1's minimal-grammar spine + P2's evidence-forced semantics + P3's divergence-census naming discipline, dropping every move a critic showed unconstitutional. The three proposals converge on a large evidence-forced core (integer bitwise; masked+trap shifts; fshl/fshr rotates; popcount; clz/ctz with is_zero_poison=false — the genuine T2 fix; bswap; imulhi-via-widen; saturating add/sub; imin/imax; a reinterpret op retiring transmute-class unsafe; fneg/fabs/fcopysign/fsqrt/floor/ceil/trunc/roundeven/fma; and comparison symmetry with fne=une). That convergence is the signal the inventory is right and largely IEEE/hardware-forced.

Resolved edges, each beating its alternatives: (1) fmin/fmax = a SINGLE dotless llvm.minimum/llvm.maximum (NaN-propagating, -0<+0, deterministic). This rejects minnum/maxnum on their CONFIRMED ±0 nondeterminism (T2/FORM-1-forced, decided now independent of the G5 FP phase), and picks propagating over quieting on an R4 ground (a NaN is surfaced, never silently swallowed) — so it does NOT reach past G5 into FP-taste, and it avoids P2/P3's .prop/.num two-mode W1 selection burden plus the llvm.minimumnum LLVM-20 version floor. A future NaN-quieting or fast sibling is FORM-1-additive (a new name), never a respelling of fmin. It is dotless for regularity with the already-shipped dotless feq/flt/fle. (2) Shifts get BOTH .wrap (mask-to-width, total) AND .trap — P1's mask-only removes the R4 trap rung and leaves ishl(x,32)=x uncatchable; rotates stay dotless because rotate-by-width is the exact identity (table-data rationale). (3) frem is named honestly as LLVM frem = C fmod (truncated-quotient, sign-of-dividend, EXACT), NOT the IEEE remainder() P1's card wrongly claimed; the round-nearest-quotient remainder() is a deferred distinct op. (4) imul.sat lowers via widen-multiply-clamp, avoiding the llvm.smul.fix.sat INT_MIN miscompile (#51019). (5) The reinterpret op is named reinterpret, not bitcast — bitcast is the literal LLVM-IR instruction spelling (R6/LEX-1 violation); reinterpret names the source invariant and pairs cleanly with cvt (bit-preserving vs value-preserving). (6) iabs.wrap's abs(INT_MIN)=INT_MIN is justified on its own terms (the two's-complement-defined edge, +2^(N-1) unrepresentable, total via is_int_min_poison=false), NOT borrowed from ineg's modular-arithmetic rationale (abs is not modular). (7) The float exact/rounding split is principled by IEEE exactness: ops that ROUND carry .strict (fsqrt, ffma; joining fadd/fsub/fmul/fdiv) and admit a future fast sibling; ops that are EXACT are dotless (fmin/fmax/fneg/fabs/fcopysign/ffloor/fceil/ftrunc/froundeven/frem/comparisons) — this resolves the P1-vs-P2 frem-suffix split. fround is spelled froundeven so the tie-mode is in the name (LEX-1: a C-prior writer must not read it as ties-away); fround.away/rint/nearbyint are cut (no v0 workload; no dynamic rounding state). fisnan and ibitreverse are cut (fne(x,x) already tests NaN; bswap has no reversal-pair workload) — R1/R3. (8) Generic numerics: an Int/Float SPLIT of lawless marker bounds, never P3's single Numeric-with-associative(add) (strict float add is non-associative and OP-3 forbids reassociation — the law is either unsatisfiable for floats or licenses illegal reassociation); 0_T/1_T are param-ONLY literals (no zero<T>()/0_i32 dual spelling). (9) Integer radix stays decimal (the committed floor: R4/frequency-safe); hex is DECLINED for v0 and routed to the P13 harness — P3's fixed-width-hex-replacing-decimal forces every count into 0x00000064 (inverts R1 frequency-optimality) and is the one unvalidated FORM-1 bet that must not freeze. Integer literals gain an inline lexical sign + range-check-on-the-signed-value (discharges D4, makes -2147483648_i32 writable); floats gain a shortest-round-trip canonical form (fixes the latent 1.50/1.5 FORM-1 hole, admits exponents). Codegen honesty is stated as table data: ffma is a correctly-rounded libcall on non-FMA hardware (accuracy guaranteed, speed conditional), and deterministic fmin/fmax is a compare-select sequence on x86 pre-AVX512 (the P0-vs-determinism tension, with minnum disqualified). All dotted rows depend on the separate Tier-0 D1 OPNAME-call-production fix; the dotless rows land immediately under GRAM-5 call:=IDENT — the natural dotless-first phasing.

### Spec changes (apply-ready)
=== [OP-6] NEW RULE (append after OP-5) ===
[OP-6] Operation-name convention (regularity, W1-predictable). (1) DOMAIN PREFIX: an arithmetic/logic/bit/compare op carries a domain prefix — `i` (integer), `f` (float), or `b` (Bool) — whether or not a cross-domain twin exists; structural/representation ops (`cvt`, `reinterpret`, `len`, `slice_of`, `box_new`, `arena_new`) carry no prefix. (2) MODE SUFFIX: a `.mode` suffix appears iff the op sits on a mode axis; single-behavior ops are dotless. The mode axes are exactly: integer result-overflow {`wrap`,`trap`,`checked`,`sat`}; shift out-of-range-amount {`wrap`,`trap`}; float rounding {`strict`}. Which axis members exist per op family is OP-2 table data. (3) Signedness-parametric lowering keyed on the already-explicit type argument (e.g. `ishr` = `ashr` if T signed else `lshr`; `imin` = `smin`/`umin`) is NOT overloading — it is the same discipline as the existing `ilt` = `slt`/`ult` row.

=== [OP-1] EXTEND the operation table (append these rows; REPLACE the existing float-comparison row as noted) ===
| op | domain | signature | effects |
|---|---|---|---|
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
| `reinterpret` | equal-width int/float pairs: {i32,u32}<->f32, {i64,u64}<->f64 | `(Src) -> own Dst` | pure |
| `fneg` `fabs` | f32 f64 | `(T) -> own T` | pure |
| `fcopysign` | f32 f64 | `(T, T) -> own T` | pure |
| `fmin` `fmax` | f32 f64 | `(T, T) -> own T` | pure |
| `ffloor` `fceil` `ftrunc` `froundeven` | f32 f64 | `(T) -> own T` | pure |
| `frem` | f32 f64 | `(T, T) -> own T` | pure |
| `fsqrt.strict` | f32 f64 | `(T) -> own T` | pure |
| `ffma.strict` | f32 f64 | `(T, T, T) -> own T` | pure |
| `finf` `fnan` | f32 f64 | `() -> own T` | pure |
REPLACE the row `| feq flt fle | f32 f64 | (T,T)->own Bool | pure |` WITH:
| `feq` `flt` `fle` `fgt` `fge` `fne` | f32 f64 | `(T, T) -> own Bool` | pure |

=== [OP-7] NEW RULE (append after OP-6): normative edge/table-data semantics + confirmed lowerings. Every totality edge is closed here as table data, so no new row is writer-reachable poison (T2/W3). ===
[OP-7] For the OP-1 rows added in this revision:
- `iand`/`ior`/`ixor` -> LLVM `and`/`or`/`xor`; `inot` -> `xor x, -1` (total; there is no dedicated NOT instruction).
- SHIFT/ROTATE amount is `u32`, zero/sign-extended or truncated to T's width in lowering. `ishl.wrap`/`ishr.wrap`: amount is masked to `amt & (width-1)` before the shift — TOTAL, defined for every amount, no poison. `ishl.trap`/`ishr.trap`: traps iff `amt >= width` (checked; elided when the amount is a proven constant/range per OP-4/OP-5). `ishr` = `ashr` if T signed else `lshr`; `ishl` = `shl`. `irotl`/`irotr` -> `llvm.fshl.iN(x,x,amt)`/`llvm.fshr.iN(x,x,amt)`, whose amount is taken modulo width by definition — TOTAL (rotate-by-width is the exact identity, so no trap mode exists).
- `ipopcount` -> `llvm.ctpop.iN`. `iclz`/`ictz` -> `llvm.ctlz.iN(x, i1 false)`/`llvm.cttz.iN(x, i1 false)`; the `is_zero_poison=false` flag makes ZERO-INPUT return the bit width (the zero-input fix; no poison). Counts return `u32` (bit positions are word-bounded < 64; container lengths are `u64` via `len` — two magnitude domains, stated once).
- `ibswap` -> `llvm.bswap.iN` (i16/u16/i32/u32/i64/u64; i8/u8 excluded — single-byte swap is identity and the intrinsic requires width a multiple of 16).
- `imulhi` -> high N bits of the full 2N-bit product: (sext if T signed else zext) both operands to 2N, `mul`, `lshr N`, `trunc` to N; ISel emits x86 MUL-high/MULX, ARM SMULH/UMULH. i64/u64 use an IR-internal i128 intermediate (not a source type).
- `iadd.sat`/`isub.sat` -> `llvm.sadd.sat`/`ssub.sat` (T signed) or `uadd.sat`/`usub.sat` (T unsigned); clamps to T's [MIN,MAX] as table data. `imul.sat` -> widen to 2N (signedness-from-T), `mul`, clamp to [T_MIN,T_MAX] via `smin`/`smax` (unsigned: `umin` to MAX, lower bound 0), `trunc`; this widen-clamp lowering is used INSTEAD of `llvm.smul.fix.sat` (which requires a scale operand and had a signed saturation miscompile at INT_MIN*INT_MIN, LLVM #51019).
- `imin`/`imax` -> `llvm.smin`/`smax` (signed T) or `llvm.umin`/`umax` (unsigned T).
- `iabs.wrap`/`iabs.trap`/`iabs.checked` -> `llvm.abs.iN(x, i1 false)` for the value; `is_int_min_poison=false` makes `abs(INT_MIN) = INT_MIN` — the two's-complement-defined edge value (the true magnitude +2^(N-1) is unrepresentable in T), TOTAL. `iabs.wrap` returns it; `iabs.trap` traps iff x==INT_MIN; `iabs.checked` returns `Err(Overflow)` iff x==INT_MIN. (This edge is justified as the defined two's-complement result, NOT as modular arithmetic.)
- `reinterpret` -> LLVM `bitcast` instruction: pure bit reinterpretation, value-changing but bit-preserving; ALL NaN payloads and sign bits are preserved exactly. It is the bit-preserving counterpart of value-preserving `cvt`.
- `fneg` -> LLVM `fneg` instruction (sign-bit flip; NOT `fsub(0.0,x)`, which is IEEE-WRONG for +-0 and NaN). `fabs` -> `llvm.fabs` (sign-bit clear; NOT select tricks). `fcopysign` -> `llvm.copysign`.
- `fmin`/`fmax` -> `llvm.minimum`/`llvm.maximum` (IEEE-754-2019: NaN-PROPAGATING — if either operand is NaN the result is NaN; -0.0 < +0.0; deterministic). `llvm.minnum`/`maxnum` are REJECTED: they return an unspecified operand on the +-0 tie (nondeterministic), which violates T2/FORM-1 reproducibility. Codegen note (table data): this is a compare-select sequence on x86 pre-AVX512 rather than one instruction — the determinism guarantee is the reason minnum's near-single-instruction form is disqualified.
- `ffloor`/`fceil`/`ftrunc` -> `llvm.floor`/`ceil`/`trunc` (roundToIntegral toward -Inf/+Inf/0; result exact; stays in the float type — distinct from `cvt` float->int). `froundeven` -> `llvm.roundeven` (roundToIntegral ties-to-even, matching the ambient default rounding of `fadd.strict`); the tie-mode is in the name to avoid the C `round()`=ties-away divergence (LEX-1). `rint`/`nearbyint` are excluded (v0 has no dynamic rounding-mode state).
- `frem` -> LLVM `frem` instruction = C `fmod`: remainder with the DIVIDEND's sign, truncated quotient; EXACT. This is NOT IEEE-754 `remainder()` (round-nearest quotient), which is a deferred distinct op.
- `fsqrt.strict` -> `llvm.sqrt` (correctly rounded; IEEE-mandated). `ffma.strict` -> `llvm.fma` (single-rounding fused a*b+c) — a DISTINCT op, not the OP-3-forbidden contraction of `fmul.strict`+`fadd.strict`. Codegen note (table data): on hardware WITHOUT an FMA unit this lowers to a correctly-rounded libcall (accuracy guaranteed, speed conditional).
- COMPARISONS: `feq`/`flt`/`fle`/`fgt`/`fge` -> `fcmp oeq`/`olt`/`ole`/`ogt`/`oge` (ORDERED: false if either operand is NaN). `fne` -> `fcmp une` (UNORDERED: true if either operand is NaN), so `fne` = `bnot(feq)` on every input including NaN. There is no separate isNaN op: `fne(x,x)` is true iff x is NaN.
- `finf` -> +Inf bit pattern (-Inf = `fneg(finf<T>())`). `fnan` -> the canonical quiet NaN (sign 0, quiet bit set, zero payload: 0x7fc00000 for f32, 0x7ff8000000000000 for f64); other NaN payloads are reachable only via `reinterpret`.

=== [OP-2] EXTEND (append, as positive table data per META-3; do not add exception clauses) ===
Mode-axis membership per family is table data: integer add/sub/mul carry {wrap,trap,checked,sat}; div/rem carry {trap,checked} (no wrap — no modular semantics for divisor zero; no sat — no clamp target); ineg/iabs carry {wrap,trap,checked} (no sat in v0 — no named workload). Shifts carry {wrap,trap} (no checked — a bad shift amount is a program-controlled decision, not data-derived overflow; masking a shift silently discards writer intent, so a trap rung is offered, whereas masking a rotate is the exact identity so rotates are dotless-total).

=== [OP-3] RESTATE (replace the sentence "v0 defines only .strict float modes ...") ===
[OP-3] Float ops that ROUND carry `.strict` (IEEE 754, no reassociation, no contraction) and are the family that a future fast-math mode would relax: `fadd.strict`, `fsub.strict`, `fmul.strict`, `fdiv.strict`, `fsqrt.strict`, `ffma.strict`. Float ops that are EXACT or exact-selection are dotless (`fneg`, `fabs`, `fcopysign`, `fmin`, `fmax`, `ffloor`, `fceil`, `ftrunc`, `froundeven`, `frem`, and the six comparisons). Approximation/fast-math modes remain an OPEN numeric-semantics question (G5); if opened, a relaxed float op is introduced as a distinct OPNAME (FORM-1-additive), and whether the dotless exact ops retroactively gain a `.strict` suffix for regularity is a FORM-1-breaking question routed to the G5 track alongside the already-dotless comparisons.

=== [FORM-5] EDIT the literal grammar ===
Replace the integer and float clauses:
- INTEGER: `int_lit := "-"? [0-9]+ "_" INT_TYPE`. The leading `-` is legal iff INT_TYPE is signed. The SIGNED value must lie in INT_TYPE's range (signed [-2^(N-1), 2^(N-1)-1]; unsigned [0, 2^N-1]); out-of-range is a hard error (fixes `256_u8`, `2147483648_i32`; makes `-2147483648_i32` = i32 MIN directly writable — discharges Tier-0 D4 for numeric literals). The `-` is lexically part of the literal, not a unary operator (unambiguous because GRAM-6 has no operators). No `-0` (canonical `0`), no leading `+`, no leading zeros. Decimal-only is retained as the committed floor (radix policy is a registered FORM-1-breaking-if-late experiment; see needs_experiment).
- FLOAT: `float_lit := "-"? [0-9]+ "." [0-9]+ ("e" "-"? [0-9]+)? "_" FLOAT_TYPE`. The canonical spelling is the UNIQUE shortest decimal digit string (fewest significant digits) that round-trips to the target IEEE value under round-to-nearest-ties-to-even, with >=1 integer and >=1 fraction digit, no redundant trailing zeros, lowercase `e`, no `+` and no leading zeros in the exponent; positional vs scientific layout is chosen by a fixed decimal-point-position threshold pinned in FORM-2 (byte-stable). `-0.0` is distinct from `0.0`. The toolchain verifies canonicity by formatting the parsed value to its shortest decimal (deterministic; Ryu/Grisu-class) and byte-comparing — fixing the latent `1.50_f64`/`1.5_f64` FORM-1 hole and admitting exponents for extreme magnitudes. (Whether non-canonical float input is REJECTED or CANONICALIZED inherits the FORM-1 P12 decision; the canonical form itself is forced now.)
- NaN/Inf are NOT literals (a spelling like `inf_f64` would lex as IDENT and collide under GRAM-1/TYPE-6); they are the nullary ops `finf`/`fnan` [OP-1].
- GENERIC-NUMERIC: `0_T` and `1_T` are legal iff `T` is a gparam bound by a numeric contract (`Int` or `Float`, PRE-1); they denote T's additive/multiplicative identity and are the ONLY generic-parameter numeric literals (0 and 1 are range-safe in every model type). Non-identity magnitudes over a parameter T are a hard error (range depends on the instance — T2/W3). These are param-only: a concrete type still uses `0_i32` etc., so there is no dual spelling. Monomorphization (FN-2) rewrites `0_T`/`1_T` to the concrete literal pre-IR.

=== [PRE-1] EXTEND the prelude (leading candidate; the exact contract-vs-predicate encoding is coupled to the FN-3/1H generics cluster — see remaining owner rulings) ===
Add two closed-conformer marker contracts with no fn signatures and no laws:
  contract Int {}    // conformers exactly i8 i16 i32 i64 u8 u16 u32 u64
  contract Float {}  // conformers exactly f32 f64
A bound `<T: Int>` (resp. `<T: Float>`) licenses, inside the body, the integer (resp. float) OP-1 rows at T and the `0_T`/`1_T` identity literals. FN-2 monomorphization re-checks each instantiation concretely, so no runtime dispatch appears. NO `associative`/`commutative`/`identity` law is attached (strict float add is non-associative and OP-3 forbids reassociation; the Int/Float SPLIT keeps any such law off floats). This lets a generic `sum<T: Int>`/dot-product/min-max form its identity and call `iadd`/`imul`/`imin` on T.

=== GRAMMAR/TOOLCHAIN NOTES (not new rules) ===
- No FORM-3 change: dotted new ops (`iadd.sat`, `iabs.wrap`, `ishl.trap`, `fsqrt.strict`, `ffma.strict`) are OPNAME tokens; dotless new ops parse under GRAM-5 `call := IDENT`. HARD DEPENDENCY: every dotted row requires the Tier-0 D1 fix (a call production that consumes OPNAME); this cluster adds NO new grammar need beyond what `iadd.wrap` already forces. Dotless-first phasing: the dotless rows are landable immediately.
- Checker: zero ownership-rule change (verified — checker.py's `call` arm only recurses into copy-primitive args). `reinterpret<Src,Dst>`/`cvt<Src,Dst>` use the existing multi-targ grammar (GRAM-3 `targs`); democ's single-targ path is a mechanical fix. democ is i32-only and discards the type arg, so signedness-from-T (ashr/lshr, smin/umin, sadd/uadd.sat) and the sign/float/other-width literal lexer are real (frontend-scale) democ work, not zero-cost emit cases.

### Draft derivation-ledger rows
| OP-1 (op-table completion: bit/shift/count/mulhi/sat/min-max-abs/reinterpret/float-math/comparison-symmetry/finf-fnan) | existence-only (mixed) | SEMANTICS evidence-selected: each row is a single machine instruction or an ISel/InstCombine-recognized pattern named to a confirmed LLVM intrinsic (P0/R0 — the O(width)-loop emulations a rejected weak writer would emit are slow and un-re-patternable by the vectorizer); every totality edge closed as table data with no writer-reachable poison (T2/W3: ctlz/cttz is_zero_poison=false, masked/trapped shift, abs is_int_min_poison=false, ordered/unordered fcmp per-op) via the N001/N002 poison model; IEEE-correctness is R4 not sugar (fsub(0,x)!=fneg, minnum ±0-nondeterministic, sqrt/roundToIntegral/single-rounding-fma IEEE-mandated — the emulations are forbidden silent corruption). SURFACE (i/f prefix, dotted spellings) inherits OP-1's existing R3-provisional surface debt. | OP-6 (naming convention) | derived | R3 (one form per need) + META-2 (no context-dependent spelling) + FORM-3 OPNAME + W1 (predictable naming); prefix/suffix inherit OP-1 surface provisionality. | OP-3 (restated: .strict=rounding ops, exact float dotless) | derived | IEEE-754 exactness classification (evidence) + R3; fast-math kept OPEN (G5). | FORM-5 (signed int + range-check; shortest-round-trip float; NaN/Inf via ops; 0_T/1_T param-only) | mixed | int-range-check derived (T2/W3 closes D4 silent-corruption); shortest-round-trip derived (FORM-1-forced unique decidable one-spelling-per-value); NaN/Inf-as-ops derived (GRAM-1 collision avoidance); decimal-only radix existence-only (P13 experiment, FORM-1-breaking-if-late). | PRE-1 (Int/Float marker contracts + generic identities) | existence-only | R2 numeric-generics need + T2/W3 (Int/Float split keeps the non-associativity of strict float off any law); exact contract encoding coupled to FN-3/1H (leading candidate, not evidence-selected).

### Rust (R0) delta
Inventory is PARITY with Rust std (wrapping_*/saturating_*/checked_*, i32::MIN, rotate_left, count_ones/leading_zeros, to_bits/from_bits, f64::sqrt/floor/mul_add) — acceptable for the same reason the ledger accepts it for TYPE-1: these primitives are hardware/IEEE-FORCED, not design choices. The R0 delta lives in the wrapping architecture, and it is real: (1) NO DEFAULT ARITHMETIC + NO DEBUG/RELEASE DIVERGENCE — Rust `<<` is debug-panic vs release-mask and `+` is debug-panic vs release-wrap; every Whitefoot node names wrap/trap/checked/sat (arithmetic) or wrap/trap (shift), one semantics in all builds, and there is no `unchecked_shl`/`unchecked_*` unsafe escape (W3). (2) DETERMINISM ON ±0 — Rust f64::min/max route through the nondeterministic minnum tie; Whitefoot commits to IEEE-754-2019 llvm.minimum/maximum (deterministic, NaN-propagating), a bit-reproducibility delta Rust does not give. (3) MORE OPTIMIZER-VISIBLE FACTS — the per-node rounding contract (.strict vs future fast), overflow-as-typed-Result the checker propagates, and single-rounding ffma as an explicit node (vs Rust mul_add's discretionary fuse) are all source-level facts rustc does not emit. (4) CHEAT-PROOFNESS — `reinterpret` is a TOTAL SAFE op retiring Rust's writer-emittable `transmute` unsafe; clz/ctz/abs/shift totality is CHECKED table data with no writer-reachable poison variant, where Rust ships the poison forms behind unsafe intrinsics. (5) LITERALS — `-2147483648_i32` is directly writable via inline sign + range-on-signed-value, where Rust needs i32::MIN because `-2147483648` parses as unary-neg of an out-of-range literal.

### Experiment spec
INTEGER RADIX A/B (registered P13 / backlog-91f; the one FORM-1-breaking-if-late literal choice). On the M3 AI-codegen harness (which does not yet exist — its absence is the meta-blocker), across W1 model tiers, measure first-emit literal correctness on a MIXED corpus deliberately balancing the frequent case (decimal counts, sizes, loop bounds) against the rare case (bit masks, flags, hash constants). Arms: {A = decimal-only + inline sign + range-check-reject (the shipped floor)} vs {B = fixed-width zero-padded lowercase hex as sole radix (digit count == width/4, so out-of-range is lexically unrepresentable)} vs {C = decimal + optional hex second radix, if FORM-1 were relaxed}. Metrics: total wrong-but-accepted literals (silent value error), first-parse acceptance rate, and repair-iterations-to-green. Decision rule: ship hex only if it strictly reduces wrong-but-accepted AND repair iterations across the mixed corpus at the weakest tier; otherwise decimal stands. Note: adopting B/C after a corpus exists is a breaking re-canonicalization of every integer literal — this is why the floor must be committed now and the experiment run before freeze.

### Remaining owner rulings (6)
1. Integer radix decimal-vs-hex (needs the M3 harness; FORM-1-breaking-if-late; decimal shipped as the committed floor).
2. Generic-numeric contract ENCODING: the Int/Float lawless marker-bound is the leading candidate, but 'a bound licenses built-in op-table rows' extends FN-3's method-contract model — route the exact predicate-vs-contract shape to the FN-3/1H generics cluster for ratification (do not freeze the contract form here; the 0_T/1_T param-literal grammar is forced-sound and ships now).
3. Bit-count return type: u32 chosen (uniform bit-position type; matches Rust count_ones/leading_zeros -> u32, divergence-census clean). Alternative T (avoids a cvt when feeding the count back into T-arithmetic) is a minor, unmeasured W1 call; changing a return column after code exists is not FORM-1-additive, so confirm u32 or flip before freeze.
4. Whether `reinterpret` also covers same-width int<->int resign (i32<->u32): coordinate with the Tier-0 D6/cvt errata cluster so resign has exactly one home (reinterpret = bit-preserving vs cvt = value-preserving) and no dual spelling; scoped here to int<->float per the brief.
5. SEQUENCING: all dotted rows (iadd.sat, iabs.wrap/trap, ishl/ishr.trap, fsqrt.strict, ffma.strict) are blocked on the Tier-0 D1 OPNAME-call-production fix (errata cluster). Confirm dotless-first phasing is acceptable, or land D1 first.
6. IEEE remainder() (round-nearest quotient) is deferred as a distinct op from frem(=fmod); confirm no v0 workload needs it now.


## Cluster: data-stack — Data: array construction, runtime-sized allocation, collections, constants

- confidence: **high** · selection_ground: mixed · form1_breaking: False · needs_experiment: True
- changed/new rule IDs: GRAM-2, GRAM-3, TYPE-2, CONST-1, CONST-2, OWN-10, FN-7, STOR-1, STOR-3, OP-1, OP-6, EFF-2

### Recommendation
Ship the intersection both critics converged on, built on Proposal 2's ladder skeleton and hardened against every objection. Structure: one type + one constructor per (when-known x where-lives) rung, no rung's spelling overlapping another's (FORM-1/R3-clean).

ADOPTED, with the specific merges the critics forced:
- array_new FILL-only (Proposal 2), NOT enumerated (P3) and NOT with a literal (P1). Fill is the unique runtime array form AND the only one that constructs generic-length arrays (`zeros<const N>() -> array<i32,N>`), so it closes D8's construction side; enumerated array_new (P3) cannot write N symbolic args and leaves generic-N unbuildable (R2 wrong cut). A runtime `[...]` literal (P1) is deleted: `[v,v,v]` and `array_new<T,3>(v)` are two byte sequences for one value (FORM-1 violation / R3 two-forms). Distinct-value runtime arrays = fill then `set index<T>` (D0: verbosity free). Empty arrays are constructible (fill with N=0), which the P1 >=1-element literal could not do.
- buffer<T>: fixed-length, NON-growable, single-owner, affine {ptr,len} heap value (all three converge here; the critics confirm it is the correct substrate and adds ZERO new lifetime reasoning — checker treats it as box).
- const items + closed NON-arithmetic const-expr sublanguage closing D8 (`const := int_lit | IDENT`).
- Proposal 1's OWN-10 "any region legal for a const-rooted place" clause grafted in, so const tables (masks/trig/CRC) can be `slice_of`-viewed and fed to SIMD consumers WITHOUT re-admitting a 'static region — closing Proposal 2's only real (e) gap while avoiding Proposal 3's region-lattice/concurrency blast radius.
- Proposal 3's explicit REJECTION of the arena-index-pool collection basis (it resurrects use-after-free as well-typed slot-recycling) carried forward into the collections-are-library ruling — the sharpest T1 call in the batch.
- FN-7 amended to admit IMMUTABLE-only const items (all three + both critics agree: immutability triggers none of FN-7's three MUTABLE-state hazards).

REJECTED per critique: P1's array literal (FORM-1/R3); P2's FLAT-1 predicate (R1 inert — every v0 element is a primitive/trivially Flat, and its named Rust delta is vacuous because Box<[u32]> already has no drop glue; the element restriction is stated directly instead, and Flat is introduced later WITH the measured struct extension); P3's enumerated array_new (D8-half-closed), P3's const-arithmetic (overloads the `.trap` OPNAMEs in const position = META-2 context-dependence + EFF-2 spurious `traps`, and couples to the still-unresolved Tier-0 D1 OPNAME-call defect), and P3's 'static re-admission (R6 blast radius).

DEFERRED to respect the PROVISIONAL §5 / D1a gate: set-drops-old and `replace` (§5 mutation, cluster 1D) are NOT folded in as "independently overdue" (P2's gate-jump). v0 buffers are construct-use-drop-only: element mutation via `set index<T>` works (copy elements, no drop), but whole-buffer reassignment/in-place growth belongs to the library collections layer, which is exactly what needs the 1D resolution. This keeps the DATA delta purely additive and leaves provisional §5 untouched until FR-reconciled.

FIXED — the T2/R4 hole ALL THREE missed: `buffer_new(n,v)` is the first op computing a runtime allocation size `n*sizeof(T)`; unchecked multiply overflow under-allocates and the fill loop corrupts the heap (no-UB break, forbidden silent corruption). New OP-6 makes the size computation trap before allocating, and EFF-2 is amended so the trap propagates honestly.

RECORDED — the shared contingency: the P0 vectorization payoff is necessary-but-not-sufficient. buffer/const ship the SUBSTRATE, but multi-slice/windowed vectorization is blunted by OWN-7 (two slices over one root overlap conservatively) and needs OWN-9 lowered to LLVM `!alias.scope`/`!noalias` metadata — both owed by the G6 vectorization-facts cluster. Realized-today is only the borrowed-whole-buffer numeric-kernel shape (democ already emits param noalias). Recorded as a hard cross-cluster link, not claimed as a delivered win.

This beats each standalone proposal: it is the only variant that is simultaneously FORM-1-clean (one construction form per need), D8-closing for the generic-array case (fill-only), (e)-complete (const-table slicing via OWN-10, no 'static), safe on the runtime-size multiply (OP-6), and honest about the G6 dependency and the unverified A0xx cards.

### Spec changes (apply-ready)
=== GRAMMAR ===

[GRAM-2] item production (add const_decl):
  item        := fn_decl | struct_decl | enum_decl | contract_decl | conform_decl | const_decl
  const_decl  := "const" IDENT ":" type "=" cvalue ";"

[GRAM-3] add one type alternative, widen const, add cvalue:
  type   := ...existing alternatives... | "buffer" "<" type ">"
  const  := "[0-9]+" | IDENT      # bare u64: decimal literal, or an in-scope integer const-param / named-const [CONST-1]
  cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]"
    # literal here = a FORM-5 numeric or unit literal (STRING excluded: STRING is doc/check-only, FORM-5).
    # cvalue occurs ONLY as a const_decl RHS (or nested in one); it never appears in runtime expr position.

Grammar determinism (GRAM-1) preserved: `const` at item head (const_decl) is position-disjoint from `const` inside `<...>` (gparam); `buffer` is a new lowercase keyword-type head like box/slice/arena; `const := digit | IDENT` is LL(1) (disjoint FIRST); in targ position a lowercase non-keyword IDENT resolves to const, uppercase to TYPEID, reserved lowercase (i8..f64, unit) to a primitive type; cvalue's `[` is in const-RHS position, disjoint from region_params `[` (fn-signature position) and from any expr-position `[` (there is none). No desugaring; every production maps 1:1 to a core-tree node.

=== TYPES / CONSTANTS ===

[TYPE-2] add to the composite list:
  `buffer<T>` (heap-owned, runtime-length, flat contiguous homogeneous storage; a {data-pointer, u64 length} value; affine single-owner; length fixed at allocation, NO in-place growth). v0 element type T must be `copy` (a primitive); affine-element buffers are DEFERRED with recorded delta (blocked on the §5 take/replace resolution, cluster 1D).

[CONST-1] REPLACE the current text with:
  "A constant-expression (usable wherever the grammar's `const` non-terminal appears: `array<T, N>` sizes and `const` targs) is EXACTLY one of: (i) a decimal integer literal `[0-9]+` (bare, u64 by position); or (ii) an IDENT naming an in-scope integer-typed const-generic parameter (GRAM-2 gparam) or a top-level integer-typed named-const item (CONST-2). The set is closed and total: no operators, no calls, no in-language computation in v0. Constant-expressions are evaluated at monomorphization (FN-2). An IDENT that resolves to a non-integer or array-typed const is a compile-time rejection (DIAG-1). This closes the const-generic forwarding path (D8): `const N` is usable as an `array<T, N>` size and forwardable as a `const` targ. Const arithmetic (operators over constant-expressions) is DEFERRED with recorded delta; when added it must carry a DISTINCT const-eval overflow-policy name and MUST NOT overload the runtime `.trap` OPNAMEs, and it is excluded from EFF-2's exhibits-traps relation."

[CONST-2] NEW rule:
  "A `const IDENT: type = cvalue;` item declares an immutable, program-lifetime, read-only static value. `type` must be const-eligible: a primitive (TYPE-1), or `array<T, N>` of const-eligible T. `box`, `buffer`, `arena`, and `slice` are NOT const-eligible (a const is pure static rodata: no allocation, no region, no drop). The `cvalue` must TOTALLY define the value (T1): a primitive-typed const takes a FORM-5 numeric/unit literal or an IDENT naming an earlier const of that exact type; an `array<T, N>`-typed const takes `[cvalue, ..., cvalue]` with exactly N entries, each of type T. The const-dependency graph (IDENT references) is acyclic and declaration-before-use (TYPE-6); evaluation is substitution + layout only. A const item is never `move`d, `set`, or `&uniq`-borrowed (immutable). It is read via `index`/`len` (copy-out for copy elements) or shared-borrowed `&'r p` in ANY region (OWN-10 const clause), so a const table may be `slice_of`-viewed and passed to a consumer. Struct/enum-typed consts are DEFERRED with recorded delta."

=== OWNERSHIP / STORAGE ===

[OWN-10] append a fourth root clause:
  "For p rooted at a named `const` item (CONST-2): any region 'a is legal — immutable static storage has program lifetime and outlives every region."

[FN-7] append (after the no-global/no-'static clause):
  "Immutable `const` items (CONST-2) are permitted and are NOT global mutable state: being read-only they (a) never erode the noalias fact base (F1 is a write-interference invariant; reads of frozen rodata add no aliasing hazard), (b) create no hidden inter-function channel (the value is source-determined in the closed unit, not a runtime channel), and (c) are Shareable-by-construction (CAP-1), pre-seeding no mutable shared state. No `'static` region is introduced: borrows of const-rooted places are governed by the OWN-10 const clause. There remains no writer-mutable global and no `static mut` analog."

[STOR-1] amend the storage-class statement and append the collections ruling:
  "... `buffer<T>` is heap-owned: its backing block is a single compiler-derived heap allocation, released by exactly one compiler-derived free at owner scope-exit (STOR-3). A `const` item (CONST-2) is immutable static storage: program-lifetime, read-only, never dropped. The storage-class set remains closed (+2: buffer heap-runtime-length, const-static-immutable); there is still no per-binding storage annotation and no default clause. Growable/keyed collections (dynamic vector, hash map, set, byte-string, text) are NEITHER storage classes NOR kernel constructs: they are future LIBRARY structures over `buffer<T>` + struct/enum + generics (a byte-string is `buffer<u8>`; a growable vector is `struct { data: buffer<T>; count: u64; }`). They are excluded from v0-kernel and RECORDED, additionally blocked on the §5 take/replace/set-drops-old resolution (cluster 1D) that in-place buffer replacement requires; the arena-index-pool ownership pattern is REJECTED as a collection basis (it resurrects use-after-free as well-typed slot-recycling). Char/Unicode text (encoding, normalization) is OUT-OF-V0, recorded."

[STOR-3] append:
  "A `buffer<T>` drop is a compiler-derived heap free, surfaced like a `box<T>` drop on every region-exit edge (reverse declaration order). `const` items (CONST-2) are never dropped (program-lifetime static storage)."

=== OPERATIONS ===

[OP-1] add two rows and extend three domains in the operation table:
  | `array_new` | `T` copy (v0: primitive); `N` a constant-expression [CONST-1] | `(T) -> own array<T, N>`  (fills all N elements with the argument; T1: every element defined) | pure |
  | `buffer_new` | `T` copy (v0: primitive) | `(u64, T) -> own buffer<T>`  (allocates a flat buffer of the u64 length; fills every element with the argument; T1) | allocates(heap), traps |
  | `len` | `slice<'r, T>`, `array<T, N>`, `buffer<T>` | `-> own u64` | pure |
  | `slice_of` | `array<T, N>`, `buffer<T>` | `&'r place -> own slice<'r, T>`  (a borrow of the whole array/buffer place) | pure |
  and extend the GRAM-5 `index<T>(p, i)` place / OP-4 domain to include `buffer<T>` (bounds-checked against the runtime length; OOB traps, SCOPE-4).

[OP-6] NEW rule:
  "`buffer_new<T>(n, v)` computes its allocation byte-size as `n * sizeof(T)` in u64 (sizeof(T) is a monomorphization-time constant). If this product overflows u64, `buffer_new` TRAPS (SCOPE-4) BEFORE allocating: an unrepresentable buffer is a contract violation, never a silent under-allocation (R4: forbidden silent corruption; T2: no-UB). This is the sole allocation-size hazard `box_new`/`arena_new` (single-T, no runtime multiply) do not have; accordingly `buffer_new`'s effect row includes `traps`. Allocation failure (OOM) is handled as by `box_new` (TCB-level, SCOPE-3), not a language trap. `array<T, N>` performs no runtime size computation (N is a constant-expression, sized at monomorphization); a monomorphized array whose size exceeds the frame limit is a compile-time rejection (DIAG-1), so `array_new` is `pure`."

=== EFFECTS ===

[EFF-2] amend the traps clause of the exhibits relation to (change only the `traps` half):
  "a body exhibits `traps` iff it contains any `.trap` op, `check`, bounds-checked `index`, OR a call to any operation or function whose effect row includes `traps` (e.g. `buffer_new`) — even if later proven away."
  (This regularizes trap propagation across calls, closing a latent gap and surfacing buffer_new's size-overflow trap; reads/writes/allocates already propagate "per the operation table.")

=== FEASIBILITY (non-normative, for the toolchain) ===
LLVM lowering (all standard, verified to parse+lower under clang 21; NO new intrinsics): buffer_new = `umul.with.overflow.i64(n, sizeof(T))` -> trap edge on overflow (reuses the existing `llvm.trap`/`unreachable` block), then allocator call carrying the `noalias` return attribute (F4), then fill (`@llvm.memset.p0.i64` for zero/byte-uniform, else a store-loop). buffer value = `{ptr, i64}`; `len` = extractvalue; `index` = `getelementptr inbounds` + `icmp ult` + branch to trap (OP-4); `slice_of` = a `{ptr,i64}` borrow view. array<T,N> = `[N x T]` alloca; array_new fill = memset or a store-loop. const scalar = `@g = constant iN v`; const array = `@g = constant [N x T] [...]`; access = GEP-into-global + load. IMPORTANT: democ.py currently emits only alloca/`{sadd,ssub,smul}.with.overflow.i32`/`llvm.trap`/param-noalias and is i32-only; buffers, arrays, const tables, GEP, memset, malloc, aggregate/global constants, and umul-overflow are ALL NEW democ lowering (standard, but not "already exercised"). Checker.py delta is additive with no calculus rewrite: buffer is another affine base (move-once, OWN-7 overlap, drop-at-scope, like box); a const item is a new always-live immutable binding kind (copy-read OK; move/set/uniq forbidden; borrow-any-region via the OWN-10 const clause); the const-expr evaluator is a tiny integer interpreter at monomorphization, outside the ownership checker. No new lifetime facts; D1a frontend-scale preserved.

### Draft derivation-ledger rows
buffer<T> (TYPE-2/STOR-1/STOR-3/OP-1) — 🟡 derived_existence_only. Chain: EXISTENCE evidence-selected on P0 (constitution names flat runtime-sized arrays as the missing vectorization substrate) + verified F1/F2/F3 (language-level non-interference over flat ranges = the payoff class) + F4 (buffer_new's noalias-return = fresh-allocation contract, VERIFIED) -> T1 (affine owned composite, OWN-1; single compiler-derived free, STOR-3; no GC/RC) + W1 (length-carrying generalization of box_new) + R1 (sole runtime-sized flat type). Selection ground: existence evidence-selected; the fixed-length/no-in-place-growth FORM is soundness/minimality-selected (growth needs §5 take/replace, deferred) -> PROVISIONAL. Note: A001/A005/A008 (phase2-arrays) NOT in verified corpus; pull through 3-0 verification before FORM-1 freeze. F4 is verified and grounds the noalias-return.

array_new fill-only (OP-1) — 🟡 derived_existence_only. Chain: T1 (total fill defines every element) + R3 (exactly ONE runtime array constructor; fill is the only form that also builds generic-N arrays -> closes D8 construction side) + W1 (miscount-proof) + D0 (distinct values via fill + set-index). FORM (fill-only, no runtime literal) minimality/W1-selected -> PROVISIONAL, needs_experiment (M3 harness). Empty array constructible (N=0), a point for fill-only.

CONST-1 rewrite (closed const-expr sublanguage) — ✅ derived. Chain: discharges CONST-1's own DEFERRED obligation + closes D8 (const N usable/forwardable) + R2 (closes the Go go:generate external-templating gap the ledger flags time-sensitive) + W1 (tiny closed surface) + W3 (no metaprogramming escape). No-operators form minimality-selected (arithmetic deferred; must not overload .trap OPNAMEs, must avoid the unresolved D1 OPNAME-call defect).

CONST-2 (const items) — ✅ derived. Chain: serve (e) constant tables (masks/trig/CRC) + T1 (cvalue totally defines) + reconciled with FN-7 (immutable rodata triggers none of FN-7's three MUTABLE-state hazards) + P0 (frozen rodata = whole-program read-only fact base; no static mut). 

OWN-10 const clause — ✅ derived. Chain: soundness — immutable static storage has program lifetime, outlives every region; enables const-table slicing WITHOUT re-admitting a 'static region (avoids region-lattice/concurrency blast radius). Grafted from Proposal 1.

FN-7 amendment (admit immutable const) — ✅ derived. Chain: derivation-driven — FN-7's three grounds are all about MUTABLE global state; immutable const triggers none (a/noalias, b/hidden channel, c/shared-state all nullified by immutability). No 'static introduced.

OP-6 (buffer_new size-overflow trap) — ✅ derived. Chain: T2 (silent under-allocation from n*sizeof(T) overflow -> heap corruption) + R4 (runtime trap > forbidden silent corruption) + SCOPE-4 (contract violation traps). The unique runtime-size hazard box_new lacks. FIX for the shared hole all three proposals missed.

EFF-2 traps-propagation amendment — ✅ derived. Chain: R4 (honest trap propagation) — a call to any traps-row op/fn exhibits traps; regularizes propagation (parallel to reads/writes/allocates) and surfaces buffer_new's trap.

collections-as-library ruling (STOR-1 clause) — 🟡 derived_existence_only. Chain: R1 (kernel Vec/Map earns nothing over buffer+generics+structs) + R3 (kernel collections = redundant forms) + STOR-1 (collections are structures over storage, not storage classes) + the arena-index-pool rejection (T1: slot-recycling = well-typed UAF, from Proposal 3). Library-not-kernel is minimality-selected -> PROVISIONAL pending M4 self-host dogfood evidence.

Cross-cluster (recorded, non-normative): DATA ships the substrate but the multi-slice/windowed vectorization payoff is gated on G6 (OWN-7 slice-overlap refinement + lowering OWN-9 to LLVM !alias.scope/!noalias metadata). Hard dependency link; not a delivered win.

### Rust (R0) delta
Load-bearing v0 delta: buffer<T> as a NON-growable, single-owner, affine {ptr,len} heap value. With no capacity field and no in-place growth there is no realloc, so a buffer's facts — fresh-allocation noalias pointer (verified F4), immutable length, stride-1 contiguity — are stable for the binding's entire lifetime and never invalidated mid-lifetime. slice_of(buffer) loops therefore carry F1-F4 non-interference + trip-count (from len) facts BY CONSTRUCTION, where Rust's Vec<T> recovers buffer-noalias only by analysis and frequently cannot: Vec::push may reallocate under &mut, invalidating derived pointers and forcing rustc to weaker aliasing assumptions through the growth-capable abstraction. Second delta: immutable-ONLY const items — no static mut, no interior-mutable statics — freeze ALL static data (whole-program 'every global is read-only') with zero writer escape, beating Rust's static mut / UnsafeCell-in-static. The literal-free construction (array_new fill + set-index) and closed non-arithmetic const-expr layer are DELIBERATELY WEAKER than Rust const-generics/const-fn — a P1/W1 simplification, acceptable because the P0 payoff lives in buffer/array LAYOUT and aliasing, not const-eval power. HONEST BOUND (recorded): this delta is necessary-but-NOT-sufficient — the multi-slice/windowed vectorization win is gated on the G6 cluster (OWN-7 slice-overlap refinement + lowering OWN-9 to LLVM !alias.scope/!noalias metadata); realized-today is only the borrowed-whole-buffer numeric-kernel shape (democ already emits param noalias). NOT claimed: Proposal 2's FLAT-1 delta (vacuous over v0-primitives — Box<[u32]> already has no drop glue) and Proposal 3's 'rustc omits noalias across calls'.

### Experiment spec
A/B on the (unbuilt) M3 AI-codegen harness, >=3 capability tiers (low/mid/high). ARM A (SHIP CANDIDATE): array construction is array_new fill-only + `set index<T>` for distinct elements; NO runtime `[...]` literal. ARM B: add a runtime array literal `[e0,...,eN-1]` as a second constructor. Tasks: N-element small-array construction, N in {3,8,16,64}, with (i) uniform values, (ii) distinct values, (iii) generic/symbolic N (`fn zeros<const N: u64>() -> own array<i32,N>`). Metrics: first-parse success rate; arity/miscount error rate; repair iterations to green; for Arm A specifically, rate of ILLEGAL `[...]` or positional-constructor emission (a signal fill-only fights writer priors). SAFETY NOTE: all array_new/literal miscounts are check-time REJECTIONS (arity vs stated N), never silent corruption — this measures W1 ergonomics/repair cost, not a T1 hazard. DECISION RULE: ship Arm A (fill-only) unless it shows materially higher unrecovered-error or repair-iteration cost than Arm B at the low tier; if so, admit the literal for distinct-value ENUMERATED arrays ONLY (fill remains the sole uniform/generic-N form) with provably-disjoint domains, preserving FORM-1. SECONDARY (lower priority): enumerated array_new(e0,...,eN-1) vs fill array_new(v) — fill-only already wins on generic-N constructibility, so this only prices distinct-value ergonomics. FORM-1 URGENCY: freezing Arm A commits canonical bytes; a later switch to Arm B is a breaking canonical-form change, so RUN BEFORE RATIFICATION.

### Remaining owner rulings (10)
1. Array-construction form (fill-only vs admit a literal): ratify only after the M3 A/B runs; FORM-1-forward-urgent (freezing fill-only commits canonical bytes).
2. buffer naming: chose `buffer` (full word, LEX-1/FORM-consistent with array/slice/box/arena; `buf` would be the sole abbreviation). Confirm against the backlogged LEX-1 divergence census.
3. set-drops-old / `replace` (§5 take/replace, cluster 1D): kept OUT of v0 (buffers are construct-use-drop-only). Owner ruling owed on whether the §5 mutation rule lands now via the D1a Featherweight-Rust reconciliation or stays deferred with the collections layer.
4. Collections scope line: whether v0 ships a reference `vec<T>` (primitive T) library, and the exact OUT-OF-V0 boundary for Map/Set and Unicode text (recorded as explicit SCOPE exclusions).
5. Index/length width fixed at u64: couples to the TYPE-1 usize/isize/index-type inventory question (FORM-1-baked) — resolve jointly.
6. Allocation-failure (OOM) policy: v0 = trap/abort parallel to box_new (out of effect row, TCB-level); revisit Result<buffer<T>, AllocError> if fallible allocation becomes a target.
7. Arena-backed runtime buffer (arena_buffer / buffer-in-'r) for bulk-free P0: deferred; couples to the arena family's zero-exemplar-card debt (STOR-4/G10).
8. Struct/enum-typed const items and Flat-struct (copy) array/buffer elements: deferred; introduce Flat as a derived predicate WITH the measured struct extension (M4 graph/CFG dogfood), not as inert v0 machinery.
9. Verify A001/A005/A008 (phase2-arrays cards) through the 3-0 adversarial process before they ground any FORM-1 freeze; F4 already grounds buffer_new.
10. Record the hard G6 cross-cluster link: DATA's P0 payoff is unrealized for multi-slice/windowed loops until G6 refines OWN-7 and lowers OWN-9 to !alias.scope/!noalias metadata.


## Cluster: surface-ergonomics — Construct fields + expression form (nest vs name) + place readability

- confidence: **medium** · selection_ground: mixed · form1_breaking: True · needs_experiment: True
- changed/new rule IDs: GRAM-8, GRAM-9, GRAM-10, GRAM-5, GRAM-4, GRAM-2, PRE-1, EX-1, DIAG-1

### Recommendation
Merge, not pick. The sound shared core (all three proposals + both critics) is: (i) named-in-DECLARED-ORDER construction with positional REMOVED; (ii) three-address / A-normal form for compound computation, genuinely P0-NEUTRAL after mem2reg/SROA and therefore decided on FORM-1/W1, not P0; (iii) prefix places kept, `index` offset narrowed to an atom, prefix-vs-postfix deferred to the P5 harness. None reintroduces a T1/T2 hole (all frontend-only, erased before IR).

On the real axis (SCOPE/uniformity) the constitution decides it: any split — proposal 1's struct-named/enum-positional, or proposal 2's construct-named/match-positional — is exactly the irregularity D0/D2 name as the enemy and reopens R4's silent-transposition hole on the unfixed side the instant a ≥2-field variant exists. So I adopt proposal 3's FULLY SYMMETRIC naming (struct + enum-variant CONSTRUCTION + MATCH binders), using proposal 3's `field: freshBinder` match form (the key insight: a fresh writer-chosen binder distinct from the field name means TYPE-6 no-shadowing is never engaged when two arms bind fields named `value`, dodging the collision proposal 2 walked into).

But I DROP proposal 3's over-reach: `borrow_expr` STAYS an atom (per proposals 1/2), so borrows-as-arguments need no pre-binding and OWN-6's call-scoped-temporary clause and the D1a-gated §5 are UNTOUCHED. Proposal 3's OWN-6 deletion is independently sound (with every borrow let-bound the two-uniq case is caught earlier at OWN-5) but it is a §5 ownership change that belongs in the FR-reconciliation gate, not a surface cluster — routed there as an owner ruling. I fold in proposal 2's one unique asset: `call := (IDENT | OPNAME)`, which fixes a pre-existing spec bug (FORM-3 lexes `iadd.wrap` as OPNAME, so the current `call := IDENT` cannot parse any operation call) — flagged as a separate housekeeping correction, not a silent rider.

Two overstatements struck per both critics: (a) the "ANF gives more optimizer-visible facts than rustc" R0 claim is ILLUSORY — every intermediate's type is already implied by the op-table signature in IR, so (ii) is P0-PARITY and stands on P1/FORM-1 alone; I do not bank any R0 win on it. (b) By contrast (i) DOES carry a real non-perf R0 delta and I name it there, not as a perf delta.

Why B (three-address) beats A (name-when-used-≥2) and C (bounded depth): A is a use-count-dependent spelling and C is a magic-number threshold — both META-2/META-3-adjacent and both leave two spellings near their boundary; B makes non-nesting a GRAMMAR guarantee (atoms in argument position) with zero writer choice. Why named-in-declared-order beats "named-iff-≥2-same-typed": the latter's spelling depends on the type's field types and flips when a field is added — context-dependent (META-2) and a FORM-1-breaking cascade. Why uniform naming (even single-payload `Some(value: x)`) despite buying zero transposition safety there: "positional for single-field, named for multi-field" is field-COUNT-dependent (META-2); uniform naming is the only regular, FORM-1-stable form, and the single-field ceremony is an accepted D0/D2 verbosity cost (verbosity is free; irregularity is the enemy).

Corpus reality (correcting proposal 1's false claim): the normative §16 EX-1 is already flat, so GRAM-9 leaves its bytes unchanged; but the democ corpus has TWO nested sites GRAM-9 rejects — ex1.wf:22 (`sign_of(v)` nested in `ieq`) and ex2.wf:14 (`imul.wrap` nested in `ieq`) — both re-cut to `let`-bindings. GRAM-10 changes §16 EX-1's two match-arm headers and ex1.wf's arms; PRE-1 respells Some/Ok/Err payload names. (i)'s W1 magnitude is UNEXERCISABLE in the current toolchain (democ parses no structs; the corpus constructs only nullary variants), so I ship the R4-forced direction now but mark its net-W1-sign needs_experiment — I do not freeze byte form on precedent alone where a choice remained, but here no byte-form freedom remains once named + declared-order + uniform is accepted (the experiment tunes magnitude, not form).

### Spec changes (apply-ready)
Apply-ready edits to spec/kernel-spec-v0.4.md. `expr` keeps its name (now the producer/rhs level) to minimize GRAM-4 churn; arguments use the new `atom` level.

=== EDIT 1 — GRAM-2 variant gains named fields (enables named variant construction + named match binders). Replace line 47 and retire the now-unused `type_list` (line 61). ===
OLD:
  variant      := TYPEID "(" type_list? ")" ";"
NEW:
  variant      := TYPEID "(" vfield_list? ")" ";"
  vfield_list  := vfield ("," vfield)*
  vfield       := IDENT ":" type
Also DELETE the now-orphaned production `type_list := type ("," type)*` (its sole user was `variant`; removing it keeps GRAM-1's 1:1 production↔node discipline).

=== EDIT 2 — GRAM-4 statements: narrow expr_stmt to a call, and make match arms named binders. Replace the two affected productions. All other GRAM-4 positions (let/set/return/try/check/match scrutinee) are UNCHANGED because they read `expr`, whose definition is refactored in EDIT 3. ===
OLD:
  expr_stmt   := expr ";"
  ...
  arm         := TYPEID "(" binder_list? ")" "=>" "{" stmt* "}"
  binder_list := IDENT ("," IDENT)*
NEW:
  expr_stmt   := call ";"
  ...
  arm            := TYPEID "(" fieldbind_list? ")" "=>" "{" stmt* "}"
  fieldbind_list := fieldbind ("," fieldbind)*
  fieldbind      := IDENT ":" IDENT

=== EDIT 3 — GRAM-5 rewrite (the core): two-level expr/atom split; named construction; three-address; OPNAME call fix; index offset narrowed to atom. Replace the entire GRAM-5 grammar block (lines 99-110) and its inline comments. ===
NEW [GRAM-5] Expressions and places:
  expr           := atom | call | construct
  atom           := literal | "move" place | place | borrow_expr
  call           := (IDENT | OPNAME) targs? "(" atom_list? ")"
  construct      := TYPEID targs? "(" fieldinit_list? ")"
  fieldinit_list := fieldinit ("," fieldinit)*
  fieldinit      := IDENT ":" atom
  borrow_expr    := "&" REGIONID place | "&uniq" REGIONID place
  atom_list      := atom ("," atom)*
  place          := pbase psuffix*
  pbase          := IDENT | "deref" "(" place ")"
                  | "index" "<" type ">" "(" place "," atom ")"
  psuffix        := "." IDENT
(Retires `arg_list`. `borrow_expr` stays inside `atom` so borrows pass as arguments unbound and OWN-6/§5 are untouched. `index` offset is now `atom`, forcing a computed offset out to a preceding `let`. `call` now accepts OPNAME — pre-existing bug fix: FORM-3 lexes `iadd.wrap` as OPNAME, which the old `call := IDENT` could not match.)

=== EDIT 4 — three new normative rules, inserted after GRAM-7. ===
[GRAM-8] Named construction. A `construct` of struct or enum-variant type K writes every declared field of K exactly once as `IDENT ":" atom`, the IDENTs equal to K's declared field names in declared order. A missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-8 and K's declared field list, naming the expected next field. There is no positional construction form; a nullary K is written `K()`. Field names are redundant-explicit facts (TYPE-5 class): checked, never chosen, never a reordering option (declared order is the one legal byte sequence). The "name only when ≥2 same-typed fields" alternative is rejected as a context-dependent spelling (META-2).

[GRAM-9] Flat (three-address) computation. Every call argument, construct field value, and `index` offset is an `atom` (GRAM-5); a `call` or `construct` in an atom position does not derive under the grammar and is a hard error citing GRAM-9. A computed value is forwarded to another operation only by binding it with a preceding `let` — stating its explicit mode and type (TYPE-5) — and referencing the binding. Nesting and let-splitting are not two spellings of one computation; there is no expression-nesting alternative (FORM-1). `borrow_expr` is an atom, so borrows passed as arguments need no binding and OWN-6 is untouched.

[GRAM-10] Named match binders. An `arm` for variant K writes every declared field of K exactly once as `IDENT ":" IDENT` (the declared field name, then a fresh binder), in declared order; a missing, extra, repeated, misspelled, or out-of-order field name is a hard error citing GRAM-10 and K's declared field list. The binder is a fresh IDENT chosen by the writer and distinct from the field name, so TYPE-6 no-shadowing is never engaged by two arms binding fields of the same name. Binder modes remain derived by OWN-13 (unchanged; not written). A nullary variant is written `K()`.

=== EDIT 5 — DIAG-1 addendum (toolchain-floor, FORM-1-safe: diagnostic content only). Append one sentence to DIAG-1. ===
ADD: A rejection whose node lies inside a nested `place` additionally renders the offending access-path segment (the specific `deref`/`index`/field step), not only the whole-place node path.

=== EDIT 6 — PRE-1 prelude respell (§15). Replace the Option and Result lines. ===
OLD:
  enum Option<T> { None(); Some(T); }
  enum Result<T, E> { Ok(T); Err(E); }
NEW:
  enum Option<T> { None(); Some(value: T); }
  enum Result<T, E> { Ok(value: T); Err(error: E); }
(Bool/Overflow/DivideByZero/NarrowError unchanged — all nullary.)

=== EDIT 7 — §16 EX-1 respell. Only the two match-arm HEADERS change (all EX-1 calls are already flat; the Neg()/Zero()/Pos() constructs are nullary and unchanged, so GRAM-8/GRAM-9 touch no EX-1 bytes; GRAM-10 changes these two lines). ===
OLD:
      Ok(v) => {
      ...
      Err(e) => {
NEW:
      Ok(value: v) => {
      ...
      Err(error: e) => {

=== COROLLARY (no rule text; corpus re-cut for the checker/democ harness, per GRAM-9) ===
ex2.wf:14  `check ieq<i32>(imul.wrap<i32>(sum, 2_i32), 20_i32) else trap "mul drift";`
   becomes  `let m: own i32 = imul.wrap<i32>(sum, 2_i32);` then `check ieq<i32>(m, 20_i32) else trap "mul drift";`
ex1.wf:22  `check ieq<i32>(sign_of(v), 2_i32) else trap "sign drift";`
   becomes  `let sd: own Sign = sign_of(v);` then `check ieq<i32>(sd, 2_i32) else trap "sign drift";`
   (this demo line was already type-inconsistent — comparing a Sign with ieq<i32> — a democ artifact; GRAM-9 only changes its structure, not its pre-existing type error).

### Draft derivation-ledger rows
| GRAM-8 | Named-in-declared-order construction; positional removed (struct + enum variant) | 🟡 existence-only (direction derived; W1 magnitude unmeasured) | R4 shift-left (PRIMARY): positional same-typed-field transposition sits at the ladder's forbidden bottom = silent corruption, unrepresentable-as-error; named-in-declared-order lifts the dominant failure mode — wrong field-order mental model — to check-reject. FORM-1/META-1: positional REMOVED = one spelling; declared-order REQUIRED = one byte sequence (free order = N! spellings). META-2: "named-iff-≥2-same-typed" rejected (field-type-dependent, flips on unrelated edits). TYPE-5: field name is a redundant CHECKED fact (same explicit-facts discipline as mandatory annotations). LEX-1/D3 census: Swift memberwise (stated rationale = transposition-prevention), C++20 designated-init (declaration-order), Zig (no positional struct init) all PASS; deliberate D3 divergence FROM Rust's reorderable+shorthand+`..base` literal and FROM Rust's positional tuple/enum payloads. R4 direction is dispositive independent of rate. | Selection ground: mixed (R4+META-2+FORM-1 force the direction; the W1 transposition-rate MAGNITUDE and single-field-label ceremony cost are evidence-gated). Unexercisable in current democ (no struct parsing; corpus constructs only nullary variants) — registered M3-harness experiment. Residual R4 frontier (correct name, swapped VALUE) is uncatchable by any type system; closable only by distinct field TYPES (newtype discipline), a library/type choice out of scope. |
| GRAM-9 | Flat three-address / A-normal form; call/construct only at expr position, arguments/fields/offsets are atoms | 🟡 existence-only (direction derived; W1 net-sign vs nesting unmeasured) | FORM-1/META-1 (PRIMARY): nesting-vs-let-split were two structural spellings of one computation; R3/FORM-1 demand one — atoms-in-argument-position makes non-nesting a GRAMMAR guarantee (strictly stronger than a nonlocal name-when-used-twice checker rule). W1: uniform `let x: T = op<T>(atom, atom);` collapses the nested-paren/argument-boundary surface (the P4/P5 hazard) and yields per-line node-path diagnostics for the write→check→fix loop (DIAG-1/G8). TYPE-5: every intermediate carries an explicit checked mode+type (no cross-statement inference). P0: NEUTRAL — mem2reg/SROA make a named local and the subexpression the IDENTICAL SSA value (democ: nested vs split select byte-identical ARM64; ex2 folds to `mov w0,#0; ret`); the "more optimizer-visible facts than rustc" claim is STRUCK as illusory (intermediate types already implied by op-table signatures in IR). OWN-6/STOR-3: named affine intermediate is single-use, immediately moved, drop elided — negligible §5 delta; primitives moot. Precedent: LLVM IR / Rust MIR / ANF (Flanagan et al. 1993) all three-address — the IR normal form put on the surface (D3 divergence from every nested-infix human language; erased before IR = no R6 backend-marriage). | Selection ground: mixed (FORM-1+regularity force B over A/C; the B-vs-nesting weak-writer error-rate and the new mistyped-intermediate error class are M3-harness-gated). Corpus-testable TODAY (only ex1.wf:22 + ex2.wf:14 change; normative §16 EX-1 already flat). Options A (name-when-used-≥2, use-count-dependent = META-2-adjacent) and C (bounded depth = META-3 magic number) rejected. |
| GRAM-10 | Named match binders `field: freshBinder` in declared order (read-side symmetry) | 🟡 existence-only (direction derived; W1 magnitude unmeasured) | R4 (read-side symmetry): destructuring transposes symmetrically to construction; positional binders of a ≥2-field variant silently transpose = same forbidden rung, lifted to check-reject by order+name checking. D0/D2 regularity: write-named/read-positional asymmetry is itself the irregularity the enemy names — full symmetry required. TYPE-6 dodge (key design): binder is a FRESH IDENT distinct from the field name, so two arms binding fields named `value` never collide with no-shadowing (this is why proposal 2's field-as-binder reading was rejected). OWN-13 unchanged (binder modes still derived, not written). | Selection ground: mixed (R4+regularity force it; magnitude gated with GRAM-8's experiment). Changes §16 EX-1 arm headers and PRE-1 payload names (declared FORM-1-urgent breaking set). Nullary variants (True()/False()/Neg()...) unaffected. |

### Rust (R0) delta
(i) Named construction — REAL non-perf R0 delta (NOT machine code): (a) W3/canonical-bytes — Rust struct literals permit free field order + field-init shorthand + `..base` (many spellings); Whitefoot admits exactly one byte sequence (declared order, no shorthand, no update), one of the constitution's canonical-bytes deltas of record. (b) R4 coverage — Rust leaves tuple structs and ALL enum/tuple variant payloads POSITIONAL (`Ok(x)`, `Some(x)`, `Point(1,2)`), i.e. admits exactly the silent same-typed transposition this fix targets; Whitefoot names EVERY product including enum payloads = strictly fewer silent-transposition holes than Rust. Parity with Rust only on named STRUCT fields; net delta is on cheat-proofness + R4, honestly not on performance.
(ii) Three-address — P0-PARITY with Rust's arbitrary nesting, and parity is acceptable precisely because this is explicitly NOT a P0 decision (democ confirms byte-identical instruction selection). The delta over Rust is P1/FORM-1/W1: a grammar-ENFORCED single spelling (Rust lets the writer choose nest-or-let and infers intermediate types), a flatter weak-writer surface, and per-line node-path diagnostics rustc does not surface at source granularity. The "more optimizer-visible facts than rustc" framing is struck — it is not a runtime delta.
(iii) Prefix places — deliberate D3 divergence from Rust (auto-deref, `p[i]`, method chains): Whitefoot keeps explicit `deref`/`index` so every load and every bounds-checked access is a named, checker-/optimizer-visible op; machine-code parity, delta on explicit-effect visibility (OP-4 bounds, OWN-9 noalias facts) and cheat-proofness (no hidden coercions). §5 UNCHANGED (borrow stays an atom; OWN-6 untouched — proposal 3's OWN-6 deletion routed to the FR gate).

### Experiment spec
Two decoupled A/B studies on the not-yet-built M3 AI-codegen harness, run jointly with the registered P4/P5 prefix-surface experiment on a shared task set and shared model tiers (≥2 low-capability tiers = the W1 target, plus one mid tier for slope). The DIRECTION of each is already constitution-forced (R4 ladder + FORM-1 + regularity); the studies measure the W1 NET-SIGN, which no current tool can produce.

STUDY 1 — construct/binder naming (also requires a democ front-end that parses structs and ≥2-field variants; neither exists today, so this is currently unexercisable).
Arm A (baseline): positional construction + positional match binders.
Arm B (proposed): named-declared-order construction (GRAM-8) + named match binders (GRAM-10).
Task set: programs that construct and destructure structs/variants with ≥2 fields, including ≥2 SAME-TYPED fields (the transposition trap).
Primary metric: silent-transposition rate = fraction of type-checking outputs whose field/binder order is semantically wrong. Secondary: overall first-try correctness; repair-loop iterations-to-green; and the competing NEW error class introduced by B — wrong/misremembered field-name rate (e.g. mislabeling Some's `value`).
Decision rule: adopt B if transposition reduction ≥ any field-name-error increase (net weak-writer error down). Direction stands regardless; the study calibrates magnitude and the single-field-label ceremony cost (R1 question).

STUDY 2 — expression form (corpus-adjacent; testable once the harness exists, on the current grammar).
Arm A: maximal nesting (call/construct may nest as arguments).
Arm B (proposed): three-address ANF (GRAM-9).
Task set: multi-operation arithmetic/logical expressions (nesting depth ≥3) and mixed compute+call chains.
Primary metric: paren/argument-boundary error rate + total first-try correctness. Secondary: mistyped-intermediate rate (the new `let`-annotation error class ANF adds); repair iterations; mean tokens/program (recorded but not penalized — D0/D2, verbosity is free).
Decision rule: B is FORM-1/regularity-forced; the study confirms B does not net-RAISE weak-writer error via the added `let` annotations. If B unexpectedly net-raises error, revisit ONLY the annotation burden (e.g. whether intermediate types may be checker-confirmed rather than writer-stated), never the flat structure.
Report combined net-sign (A vs B on total-correct and repair-iterations) per study and jointly with P4/P5.

### Remaining owner rulings (7)
1. R0 READING (both critics flag): this cluster satisfies R0's 'name a delta over Rust' ONLY on non-performance axes (canonical-bytes/W3, cheat-proofness, R4 coverage, W1 surface) — never machine code. If R0 is read as strictly P0, the whole cluster 'fails R0' and is a P1-only decision. Needs an explicit ruling.
2. OPNAME-in-call (`call := (IDENT | OPNAME)`, EDIT 3): confirm this is an accepted housekeeping correction of a pre-existing spec bug (FORM-3 lexes `iadd.wrap` as OPNAME, unmatchable by the old `call := IDENT`) folded into the GRAM-5 rewrite, not a silent rider.
3. FREEZE (i) NOW under FORM-1 despite its W1 magnitude being unexercisable in current democ? Recommendation: YES — once named + declared-order + uniform is accepted (each R4/META-2/FORM-1-forced) no byte-form degree of freedom remains; Study 1 tunes magnitude, not form. Owner confirms the freeze vs waiting for the harness (waiting is itself a FORM-1-urgent cost: every later change is a breaking canonical-byte edit).
4. EXTEND named args to user FUNCTION calls? User-fn params carry the identical from/to transposition hazard; op-table calls (`iadd.wrap(a,b)`) likely gain nothing from lhs/rhs labels and are highest-frequency. Candidate: named args for user fns, positional for the closed op inventory — but is the head-token partition (user-fn IDENT vs op OPNAME) a clean regularity partition or context-dependent? Cluster scoped only `construct`; FORM-1-additive-then-breaking, so rule before corpus freeze.
5. OWN-6 DELETION (proposal 3, NOT adopted here): with every borrow let-bound the two-uniq-alias case is caught earlier at OWN-5, vestigializing OWN-12's call-frame logic and discharging a ledger-flagged untested clause — an independently SOUND §5 simplification, but a semantic ownership change that belongs in the FR-reconciliation gate, not this surface cluster. Route separately if wanted (requires a checker revalidation pass). This synthesis keeps borrow_expr an atom and §5 untouched.
6. NO-DEAD-BINDING reject? Three-address makes a dead pure `let` representable; EFF-3 retains unused pure calls (no termination checker). A mechanical-fix check-reject (R4 shift-left) is attractive but interacts with EFF-3/termination — out of scope for this cluster; flag before adopting.
7. STRICT-ANF verbosity: under GRAM-9 even a nullary construct passed as an argument (`f(Neg())`) must be a named `let` first. Recommendation: keep strict (any 'all-atom constructs may nest' carve-out is a context-dependent atom/rvalue distinction, META-2; verbosity is free, D0/D2). Owner may weigh a zero-arg-construct-as-atom carve-out; recommendation is strict.


## Cluster: conditional-complex — no-if / Bool-enum / statement-match / helper-idiom (FORM-1-urgent, coupled)

- confidence: **high** · selection_ground: mixed · form1_breaking: True · needs_experiment: True
- changed/new rule IDs: GIVE-1, GRAM-7, GRAM-4, GRAM-6, OWN-13, FORM-3, EX-1

### Recommendation
ADOPT (provisionally, needs_experiment) "contained let-initializer value-match with an explicit `give` terminator" — the merge both critics converged on: Proposal-1's containment MECHANISM (a real grammar production, not P3's unsound FORM-2 trick, not P2's fully-general always-expression), fixed at its one META-2 wrinkle exactly as the implementation critic prescribed, plus P2's decoupling framing and pre-registered weakest-tier silent-wrong experiment, plus P3's LEX-1 keyword census.

WHAT IT DOES. `match` gains exactly one value position: the initializer of a `let`. `let x: own T = match e { arm+ }` where every arm terminates in one `give v;` (v of the let's declared mode+type) or diverges (return/break/trap). Everything else is UNCHANGED: statement-`match` keeps its arms give-free (the dead one-sided-guard arm stays `False() => {}`), `return`-position conditionals keep return-from-arms, PRE-1 (Bool enum) and FORM-5 (no bool literals) are untouched. This deletes the GRAM-7 helper-function idiom — the named W1 pain-2 — with the smallest regular delta and the strongest R4 posture (match never enters arbitrary expression position, so deep match-in-argument nesting stays grammatically UNREPRESENTABLE, not merely check/format-rejected).

WHY IT BEATS THE ALTERNATIVES AFTER CRITIQUE. The record's three proposals converge on ONE settled primitive — explicit `give` + a check-time give-completeness pass — and the same three rejections (tail-expr arms = META-2 semicolon-distinguishes-value + FORM-1 dual-byte; dedicated branch/if = GRAM-6 no-if + R3 second conditional form; bool literals = R3 two spellings of True()/False()). So the only live question is CONTAINMENT, and the critics proved each proposal's containment story defective as written: P1's give/statement partition looked like context-dependent meaning; P2's "every arm delivers ⇒ value-match else unit-match" is a defaulting+inference rule (META-2) and is internally contradictory on mixed arms; P3's uniform `give unit;` WORSENS pain-1 and its FORM-2 nesting bound is unsound (multi-line let-init matches already exist, so FORM-2 does not forbid `f(match ...)`).

The constitution critic named the real axis as a TRILEMMA: you cannot simultaneously have (a) no position-dependent give rule, (b) no `give unit;` tax on effect arms, and (c) no arm-content inference — P1 pays (a), P2 pays (c), P3 pays (b) — and asked whether a fourth corner exists. THIS RECOMMENDATION IS THAT FOURTH CORNER, and it escapes the trilemma by a move neither critic fully spelled out: contain value-production to a distinct GRAMMAR PRODUCTION (the `let`-initializer `match_block`), so value-vs-statement is resolved production-locally — exactly the FORM-6 precedent ("disjoint productions, resolution is production-local, not contextual"), which is verbatim in the current spec — rather than by reinterpreting one production by surrounding context. Then: (a) give MEANS the same thing everywhere it is legal; its LEGALITY is checker-scoped precisely as `break`'s enclosing-loop rule (TYPE-6) — legality-by-context is not meaning-by-context, and break already establishes it, so META-2 is clean; (b) statement-match arms are give-free and UNCHANGED, so pain-1 is not regressed and no `give unit;` tax appears in real code; (c) the match's value type is STATED at the let binder (TYPE-5, never inferred) and value-consumption is production-determined (never inferred from whether arms give). The pathological `let x: own unit = match ...` requires `give unit;` uniformly (total, no exception clause, never written in practice). The implementation critic independently endorsed exactly this fix ("state give-completeness as one rule, derive the let-init-vs-statement split as declared-type = mode T for a let RHS = unit otherwise, cross-referencing the break/TYPE-6 positional precedent") and rated this containment SOUND and the leading engineering choice (smallest checker/democ delta; deep nesting unrepresentable). The constitution critic's own alternative (P2-style give-always-available) was shown by that same critic to fall back into the trilemma; the production-local containment is the only corner that pays none of (a)/(b)/(c).

HONEST SCOPE. This resolves pain-2 (helper-fn) fully and leaves pain-1 (dead guard arm) and Bool-representation deliberately OPEN, correctly routed to the harness as decisive sub-arms — the cluster is half-closed by reasoning and half-closed by measurement, stated as such, not dressed as full resolution. Value-match is orthogonal to Bool representation (it applies identically to Result/Option/Sign/Bool), so the urgent value-case fix lands now without touching PRE-1/FORM-5; only the primitive-bool+if+literal PACKAGE is truly coupled, and it is decoupled from and does not block this fix.

### Spec changes (apply-ready)
All edits are against spec/kernel-spec-v0.4.md. Line-refs are for the applier.

=== EDIT 1 — GRAM-4 (§3), statement inventory: add give_stmt, factor match_block, widen let_stmt. Arm shape UNCHANGED. ===
REPLACE the block (currently lines 80-95):
```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | match_stmt | try_stmt
try_stmt    := "let" IDENT ":" mode type "=" "try" expr ";"
let_stmt    := "let" IDENT ":" mode type "=" expr ";"
```
WITH:
```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | match_stmt | try_stmt
             | give_stmt
try_stmt    := "let" IDENT ":" mode type "=" "try" expr ";"
let_stmt    := "let" IDENT ":" mode type "=" ( expr ";" | match_block )
give_stmt   := "give" expr ";"
```
And REPLACE `match_stmt := "match" expr "{" arm+ "}"` (currently line 92) WITH:
```
match_stmt  := match_block
match_block := "match" expr "{" arm+ "}"
```
(arm, binder_list UNCHANGED.) GRAM-1 determinism note (append to the GRAM-1 rationale or GRAM-4 prose): after `"let" IDENT ":" mode type "="`, one-token lookahead resolves the three branches — `try` -> try_stmt, `match` -> let_stmt with match_block (self-terminating on `}`, no trailing `;`), otherwise -> let_stmt with `expr ";"`. FIRST(match_block)={`match`} is disjoint from `try` and from FIRST(expr); this reuses the existing try/let post-`=` disambiguation, adding one keyword branch. match_stmt and the let-initializer share the ONE match_block node kind (META-1 1:1 preserved).

=== EDIT 2 — GRAM-6 (§3, line 112), cross-ref only ===
REPLACE "Conditional control is `match` on prelude `Bool` [PRE-1]; iteration is `loop` + `break`."
WITH   "Conditional control is `match` on prelude `Bool` [PRE-1]; a conditional value is a `let`-initializer `match` [GRAM-7]/[GIVE-1]; iteration is `loop` + `break`."

=== EDIT 3 — GRAM-7 (§3, line 114), REPLACE ENTIRELY ===
[GRAM-7] `match` has one arm shape (`{ stmt* }`, [GRAM-4]) and appears in two disjoint productions sharing one core-tree node kind [META-1]: as a statement (`match_stmt`) and as the initializer of a `let` (`let_stmt`, via `match_block`). Which production a given `match` parses under is production-local, not contextual — the [FORM-6] precedent. A `let`-initializer `match` is value-producing: on every control path each arm delivers the binding's declared `mode type` by terminating in one `give e;` [GIVE-1] or diverges (`return`/`break`/trap). A statement `match` produces no value; its arms act by effect and complete without one. `return`-position conditionals deliver by returning from arms; there is no helper-function conditional-initialization idiom, and value-production is confined to the `let`-initializer, so a `match` never occupies an arbitrary expression position.

=== EDIT 4 — NEW RULE GIVE-1 (§3, insert immediately after GRAM-7) ===
[GIVE-1] `give e;` delivers `e` as the value of the arm of the nearest enclosing `let`-initializer `match`; `e` must have that `let`'s declared `mode type` [TYPE-5: the type is stated at the binder, never inferred from arms]. `give` is legal only inside a `let`-initializer `match` arm — a checker-scoped restriction exactly as `break`'s enclosing-loop rule [TYPE-6]: the grammar admits `give_stmt` and the checker restricts it, so `give`'s legality (not its meaning) depends on the enclosing construct [META-2 clean by the break precedent]. On every control path a `let`-initializer `match` arm terminates in exactly one `give e;` or diverges; a give-free path, a statement following a `give` in the same block, and a second `give` on one path are each a hard error citing GIVE-1 — the value analog of match exhaustiveness [ERR-2]. Give-completeness is a structural last-statement recursion (an arm delivers iff its final statement is `give`, `return`, `break`, or a nested value-`match` all of whose arms deliver), strictly simpler than the ownership checker [D1a]. `give e;` moves or copies `e` per [OWN-1]; a borrow-typed `e` is judged for regions exactly as a returned borrow of the same mode [OWN-4].

=== EDIT 5 — OWN-13 (§5, line 158), APPEND one clause (no §5 restructure) ===
After "...Binder modes are derived by this rule, stated once; they are not written." APPEND:
" A `let`-initializer `match` binds its value from arm `give`s [GIVE-1]; scrutinee treatment and binder-mode derivation are unchanged. Each arm delivers a value of the `let`'s declared mode+type, so on the taken arm an `own` result is moved exactly once (no double-move; T1 preserved). A `give e;` whose `e` is a borrow reaching through a binder or an outer borrow obeys [OWN-4]/[OWN-5] exactly as a returned borrow of the same mode (PROVISIONAL: this arm-result region join is an additive reuse of the existing return-of-borrow judgment and must be confirmed against the formalized calculus before §5 ratification, per D1a)."

=== EDIT 6 — FORM-3 (§2, line 25), reserve `give` ===
`give` joins the grammar-terminal reserved words (like `move`, `set`, `break`, `match`): no IDENT may spell it. (This is the existing IDENT-vs-terminal partition; recorded here as an explicit delta line. No keyword table exists to edit.)

=== EDIT 7 — PRE-1 (§15) and FORM-5 (§2): UNCHANGED (recorded explicitly as part of the delta) ===
PRE-1 keeps `enum Bool { True(); False(); }`. FORM-5 keeps "no boolean literals; the canonical Bool values are the constructors True()/False()". These are deliberately NOT flipped; the Bool-representation and dead-guard axes are routed to the harness (needs_experiment).

=== EDIT 8 — EX-1 (§16, lines 293-332), RE-CUT (normative bytes change) ===
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
      Ok(w) => {
        give w;
      }
      Err(e) => {
        return unit;
      }
    }
    check ieq<i32>(v, 42_i32) else trap "arithmetic drift";
  }
  return unit;
}
```
Notes for the applier: (i) `sign_of` KEEPS return-from-arms unchanged — value-match is reserved for the let-init-then-continue need, demonstrating R3 non-redundancy; (ii) the `let v: own i32 = match ... { ... }` initializer is a braced construct and self-terminates on `}` (NO trailing `;`), consistent with loop/region/match; the following `check` is the next statement; (iii) the `Err` arm delivers by diverging (`return unit;`) — GIVE-1's give-or-diverge; (iv) `deref(p)` (reading the i32 through the shared borrow `p`) is an ORTHOGONAL Tier-0 erratum in the current EX-1 (which passes `p` directly); it is fixed here only so the re-cut typechecks and MUST NOT be read as closing that separate errata item.

=== DELTA COUNT (META-5) ===
rules +1 (GIVE-1); rules edited 3 (GRAM-4, GRAM-6, GRAM-7-replaced); rules touched-additive 1 (OWN-13 clause); FORM-3 reserve-word +1; grammar productions +2 (give_stmt, factored match_block) with let_stmt widened; new arm shapes 0; exceptions +0; spellings +1 (`give`); PRE-1/FORM-5 unchanged (recorded). R3 net: conditional-construct COUNT unchanged (exactly one `match`); redundancy 0.

### Draft derivation-ledger rows
GRAM-7 (rewritten) | one match form, two productions, value-match confined to let-init | status: derived-PROVISIONAL (was existence-only on the R3-disqualified 'preserve one arm shape/cheapest' ground) | chain: R1 (let-init-then-continue is the sole conditional need with no non-helper form: arms cannot bind an outer name past TYPE-6 scope, return/break exit the construct) + R3 (exactly one conditional construct `match`; value capture reuses `let`; no redundancy — return/set conditionals keep arm-terminator forms) + W3 (give-completeness cannot be silenced) + R4 (give-completeness = check-time reject, value analog of ERR-2) + META-2 clean via FORM-6 production-local + break/TYPE-6 checker-scoped-legality precedents; closes the META-6 orphan-risk by replacing the disqualified ground | selection ground: minimality+regularity-selected -> R3-PROVISIONAL, needs_experiment.

GIVE-1 (new) | explicit value terminator + give-completeness | status: derived-PROVISIONAL | chain: R4 (shift-left: missing delivery is a loud reject, not silent unit) + W3 (delivery cannot be silenced; no tail-expr hiding place) + W1 (give is one more terminator of the return/break class; no new arm shape; no semicolon-distinguishes-value hazard) + META-2 (context-dependent legality, not meaning, via the break precedent) + D1a (structural last-statement recursion, strictly below the ownership checker) | selection ground: minimality/regularity-selected (chosen to preserve META-2) -> provisional.

OWN-13 (appended clause) | give-of-a-borrow arm-result region join | status: existence-only -> additive-PROVISIONAL | chain: T1 (single arm executes -> own result moved once, no double-move) + OWN-4/OWN-5 reuse (give-of-borrow judged exactly as returned borrow) | selection ground: derivation-forced (additive reuse); PROVISIONAL pending FR/Featherweight-Rust reconciliation of the arm-result region join (§5 gate; arena/region family has zero exemplar cards).

EX-1 (re-cut) | normative bytes | status: existence-only (unchanged status) | chain: W1 spec-primary pedagogy + W3 canonical-byte exemplar; now demonstrates value-match (main) while sign_of keeps return-from-arms (R3 non-redundancy); deref(p) folded only to typecheck, flagged as orthogonal Tier-0 errata | selection ground: follows GRAM-7/GIVE-1.

GRAM-4/GRAM-6 (edited) | additive productions + cross-ref | status: existence-only residue unchanged (loop-form/arm-shape register items untouched) | selection ground: additive.

### Rust (R0) delta
P0 lowering: PARITY with Rust on the construct — a `let`-initializer value-match lowers to a conditional branch on the discriminant (`br i1` / discriminant-compare ladder, both already emitted by democ for Bool/Result/enum matches) with each `give v` becoming a store to the let's alloca slot at the join, promoted to a `phi` (or folded to `select` for cheap pure two-arm cases) by mem2reg/SimplifyCFG — all core LLVM IR / standard passes, no intrinsic. This parity is HONEST and acceptable because the cluster is a W1/R3/W3 surface-form defect, not a P0 decision: the language's R0 P0 deltas of record (per-node numeric modes OP-1, region-explicit borrows §5, exact effect rows §9, proven-else-checked OP-4) are untouched and remain its performance justification. The deltas this fix DOES claim over Rust are W3/R3/W1, not P0: (1) Rust spreads scalar conditionals across FOUR forms (`if`, `if let`, `match`, `let ... else`); Whitefoot keeps ONE conditional construct with ONE value terminator (`give`). (2) Rust's arm/block value is the last un-semicoloned expression — a context-dependent, edit-fragile channel where moving a `;` silently changes a block's value; Whitefoot's `give` is explicit, checker-verified, canonical-bytes, so the moved-semicolon and forgot-to-return-a-value classes are unrepresentable (W3). (3) give-completeness is a value analog of exhaustiveness that Rust lacks, and Whitefoot's exhaustiveness has no wildcard/`if let` escape, so a weak or cheating writer cannot silence it. Secondary intra-language note (vs the current helper-fn baseline, NOT vs Rust): the value-match let-slot is mem2reg-promoted UNCONDITIONALLY, whereas the helper's identical phi is recovered only if the cost-gated Inliner fires — so value-match removes an inliner dependency for the most common conditional-value pattern. This is real but small, narrow (only large/multi-use conditional-value helpers the inliner declines), and unmeasured; the constitution critic correctly deflated P3's stronger 'measurably below Rust / R0 failure' framing (a small single-use helper IS inlined at -O2), so the claim is stated as conditional-gain-over-helper, parity-with-Rust.

### Experiment spec
Requires the unbuilt M3 AI-codegen harness + a reference interpreter for differential testing + the canonical parser. This ratifies the R3-provisional form and settles the two deferred axes.

ARMS (spec slice held constant except the arm-defining rules): A0 = keep-as-is (helper-fn conditional-init; statement-only match) [null baseline]; F = contained let-init value-match with `give` [LEADING]; D = general match-as-expression (match in arbitrary expr position) [tests whether the extra freedom earns its weaker R4 posture + larger checker surface — since F's own EX-1 already names every intermediate, the practical gap narrows to return/set-position and inline give-of-match]; C = tail-expr value arms, no `give` [tests the semicolon-distinguishes-value hazard]. DECISIVE SUB-AXES (the trilemma/pain-1/Bool questions the constitution cannot settle by reasoning): Guard sub-axis {G-light = dead one-sided arm stays `False() => {}` give-free [F's choice] vs G-branch = dedicated two-arm branch construct} — pain-1; Bool sub-axis {B-enum = Bool enum + match + True()/False() [PRE-1/FORM-5 as-is] vs B-prim = primitive bool + if + literals} — run INDEPENDENTLY of the value axis since value-match is orthogonal to Bool representation.

CORPUS: each task = signature + doc + expected behavior + a democ-executable reference oracle, engineered to hit the pains: (i) one-sided loop break-guard [pain-1]; (ii) local conditional-init-then-continue [pain-2, F's target]; (iii) nested 3-way classification [sign_of shape]; (iv) a conditional value flowing into a later call; (v) a conditional value that is a BORROW [OWN-13/§5 region-join stress]; (vi) a conditional value that is a Result/Option [democ symbolic-pair/ptr materialization stress].

MODEL TIERS: >=3 W1 capability tiers including a genuinely weak tier (~7-8B); W1 mandates the WEAKEST tier that clears a pre-registered floor decides.

METRICS: (1) first-parse-success under the canonical grammar [exposes C/D semicolon+nesting errors vs F's regular give]; (2) first-check-accept [exhaustiveness ERR-2 + give-completeness GIVE-1 + ownership]; (3) PRIMARY = silent-wrong-rate via differential testing = checker-accepted-but-behaviorally-divergent on a random input battery vs the reference impl [the R4-forbidden mode: swapped True/False arm, wrong give value, off-by-one guard]; (4) repair-iterations to accept-and-pass under rule-cited diagnostics [write->check->fix loop, G8]; (5) secondary non-gating: tokens [D2a measured-not-gating] and nonlocal-helper count [A0's structural tax].

DESIGN: within-task/between-arm, randomized order, blind automated grading (canonical parser + differential exec, no human). Pre-register the silent-wrong PRIMARY endpoint; report Wilson/bootstrap CIs per (task x arm x tier).

DECISION RULE (pre-registered; BALANCE: all value-axis arms lower to br+phi/select so P0 is equal, W1 at the weakest tier is the sole tiebreaker): minimize weak-tier silent-wrong-rate FIRST (F must NOT regress vs A0, else disqualify), then weak-tier first-check-accept, then repair-iterations. Guard sub-axis: adopt G-branch ONLY on positive evidence that the give-free dead arm drives weak-tier parse/silent-wrong errors (absent that, G-branch is a premature R3 second form). Bool sub-axis: if B-prim wins, PRE-1/FORM-5/GRAM-6 flip together (the true FORM-1 coupling), decoupled from and not blocking F.

HYPOTHESES: F beats A0 on repair-iterations + nonlocal-count with no silent-wrong regression; F beats C on first-parse-success (no semicolon hazard) and beats D on silent-wrong (D's nesting raises arm-polarity/target-resolution errors).

### Remaining owner rulings (5)
1. FR/Featherweight-Rust reconciliation (§5 D1a gate, BLOCKING before §5 ratification): confirm the OWN-13 arm-result region join for give-of-a-borrow is covered by the existing return-of-borrow argument (expected trivially yes; the clause is additive and §5 is unrestructured). T1/T2 hold only conditional on this; the arena/region family has zero exemplar cards, so do not assert on intuition.
2. `give` keyword spelling (give vs value vs be; `yield` REJECTED by the LEX-1/D3 census for its coroutine/generator prior — a genuine semantic mismatch; Zig `break :blk v` rejected for overloading loop-only break/coupling to labels; Rust/Swift tail-expr rejected for breaking `;`-termination + META-2). Leading = `give`. FORM-1-urgent-secondary: pin the spelling before any corpus freeze.
3. Admit `set p = match { give }` for uniformity with let-init, or exclude it (my choice) to avoid R3 redundancy with set-from-arms? Revisit only if the A/B shows weak writers reach for set-match; if so, prefer generalizing the binding target over adding a form.
4. Pre-existing FORM-1 residue surfaced (NOT opened here): the v0.4.1 scrutinee-widening makes both `let s = expr; match s { }` and `match expr { }` legal when s is unused — a canonical-uniqueness question independent of this cluster; flag for a separate ruling. EX-1 inlines the scrutinee per the v0.4.1 no-mechanical-temporary intent.
5. Forward-compat (expected trivially yes): confirm `give` generalizes unchanged if `match` later gains integer/dense-switch arms (dossier P15) and decide, before both freeze, whether `give` unifies with a future value-carrying `break` (Zig unifies both) or stays match-only.


## Cluster: memory-expressiveness — Region-generic structs, graphs, shared ownership, interior mutability

- confidence: **medium** · selection_ground: mixed · form1_breaking: False · needs_experiment: True
- changed/new rule IDs: GRAM-2, GRAM-3, TYPE-2, OWN-1, OWN-3, OWN-6, OWN-9, OWN-14, OWN-15, OWN-16, OP-1, STOR-1, STOR-3, CELL-1, CELL-2, CELL-3, LEX-1

### Recommendation
Merge, not pick. The three proposals share a correct core (close the 1E latent defect with region params + a loud reject; uphold the STOR-3 RC ban and route sharing to a pool; put interior mutability in the gated §14 family so §5 ratifies at three modes; add replace/swap), but each ships at least one thing a critic demonstrated is unconstitutional, and ALL THREE omit the two load-bearing soundness obligations. The synthesis takes the best carded piece from each and fixes both holes.

Chosen pieces and why they beat the alternatives after critique:
- GRAMMAR from Proposal 1: reuse the existing `[...]` region_params on struct/enum (0 new spellings, FIRST-sets `<`/`[`/`{` disjoint after TYPEID, GRAM-1 intact). Beats Proposal 3's `gparam += REGIONID`, which the critics showed makes region-declaration have two spellings (`<...>` on aggregates vs `[...]` on fns) — the exact irregularity META-1/META-2 name as the enemy.
- CONFINEMENT from Proposal 3 (OWN-15), grounded letter-for-letter in FR Def 3.21 Well-Formed-Type (confirmed in fr-reconciliation-m0.md OBL-0 pt 2), NOT Proposal 1's "phantom-borrow, 0 new judgments" fiction, which critic 1 showed understates the build (multi-field holder resolution + a self-locking uniq-field problem) and mis-claims zero §5 delta.
- GRAPH SUBSTRATE decoupled from region-generic aggregates (implementation critic's missing option): a heap-owned single-owner `pool<T>` + region-FREE copy `handle<T>`. Region-free handles (Proposal 2's ergonomics win) mean a self-referential `struct DNode { next: own Option<handle<DNode>>; ... }` needs NO region threading — sidestepping Proposal 3's `Node<'r>` W1 burden. The handle is TYPED (Proposal 2/3), never Proposal 1's bare u64 (which critic 1/2 showed is an R4/W1 regression: index/len/foreign-handle confusion becomes an in-bounds silent wrong value).
- KEEP arena/STOR-4 (reject Proposal 2's deletion): critics showed the deletion is a minimality cut (R2-forbidden), form1_breaking, and forfeits a derived R0 delta of record (guaranteed region residency). pool is ADDITIVE alongside arena; both earn their place (arena = heterogeneous region-bump backing; pool = homogeneous growable indexed graph substrate, itself one buffer freed once, so the en-masse-free P0 story is preserved).
- CELL gated OUT of the writer grammar (Proposal 3), not in GRAM-3 (Proposal 2's leak, which critic 1 showed lets a kernel `let c: cell<i32>` parse — a W3/SCOPE-1 violation).
- DROP Proposal 3's cross-pool "region-branding = generativity" claim, which critic 1/2 proved false (a lexical region holds many pools; the brand buys no per-pool identity and the generational trap only nondeterministically fires). We SHIP WITHOUT a cross-pool-identity guarantee and state honestly that cross-pool misuse of a same-T handle is a memory-SAFE logic bug (in-bounds wrong value or a trap) outside T1/T2; the only sound shift-left (true generativity branding) may bust D1a and is census-gated, not claimed. handle<T>'s type parameter still stops handle<Node>-on-pool<Instr> (a type error), so it strictly beats bare u64.
- APPEND-ONLY is the v0 default (T1-cleanest: no slot recycle ⇒ the round2 "well-typed slot-recycling UAF" is unrepresentable, and access is a bare bounds check, no per-access generation tax). Generational per-element free is a census-gated opt-in, resolving critic 2's "mandatory generational is a P0 tax" objection.

The two omitted soundness obligations, now supplied (this is what makes it ratifiable vs the three as-written):
- OWN-14 RESULT REBORROWS: the missing §5 rule. A returned `&'r T`/`&uniq 'r T` whose `'r` comes from exactly one borrow-argument reborrows that argument (FR *w reborrowing at the call boundary; singleton-provenance preserved). This is what actually makes pool_at's element borrow overlap the pool, so pool_push's required `&uniq 'r pool` conflicts under OWN-5 — realloc-on-grow can never dangle a live element borrow. Without it, verified against checker.py, push-while-borrowed passes the checker (a UAF).
- OWN-16 replace/swap DOMAIN = region-free borrow-free owned T (table data, META-3). Forced by T-A: allowing borrow-typed T would let swap exchange two borrows a flow-insensitive checker cannot track, reintroducing the borrow-reassignment T-A bans.
Plus an OWN-6 reborrow-through-holder addend so `&uniq 'r deref(p)` at a call arg is legal (the pool snippets otherwise cannot pass `&uniq p` twice — p is affine).

Honest scope: this closes the CONSTRUCTIBILITY and SOUNDNESS gaps and gives the M4 dogfood a writable CFG/dom-tree/worklist substrate. It does NOT resolve the W1 writability costs (whole-pool exclusivity forces per-element regions; idiom selection; reborrow-vs-move) — no M3 harness exists to measure them — nor the pool-growth dependency on cluster 1C. Those are flagged needs_experiment, not faked.

### Spec changes (apply-ready)
== GRAM-2 (edit 2 productions; region_params from line 58 reused verbatim) ==
struct_decl := "struct" TYPEID generics? region_params? "{" doc? field* "}"
enum_decl   := "enum" TYPEID generics? region_params? "{" doc? variant* "}"
(FIRST after TYPEID: "<"(generics) / "["(region_params) / "{"(body) — pairwise disjoint; GRAM-1 two-token determinism unaffected. Use site: existing `type := TYPEID targs?`, `targ := type | REGIONID | const` already admits region args.)

== GRAM-3 (edit `type`; add two writer-emittable keyword type-formers; cell is NOT here) ==
        | "pool" "<" type ">" | "handle" "<" type ">"

== TYPE-2 (addend) ==
`pool<T>` (growable single-owner indexed collection) and `handle<T>` (copy non-owning index) join the composite inventory. `arena<'r,T>` is RETAINED — not subsumed; its region-bump backing and STOR-4 confinement differ from the heap pool.

== OWN-1 (copy-class addend) ==
The copy class gains `handle<T>` (morally a primitive index; copying it duplicates no ownership). No structural-copy rule is added: aggregates remain affine (FR Def 3.6); a nullable neighbour is `own Option<handle<T>>`, read without consuming its node by matching through a shared borrow.

== OWN-3 (edit one clause) ==
"Region identifiers are unique within their declaring item (a function OR a region-generic struct/enum)."

== OWN-6 (reborrow-through-holder addend) ==
A `borrow_expr` whose place resolves (OWN-6) through a live borrow's holder is a REBORROW of that holder's borrow: it is permitted despite OWN-5 against the holder's own borrow (the sanctioned "through the holder" access); the reborrow is `&uniq` only if the holder's borrow is `&uniq`; for the reborrow's lifetime the holder binding may not be otherwise accessed. Singleton provenance (T-A) is preserved: the reborrow's provenance is the holder's single borrowed place. (This makes `&uniq 'r deref(p)` legal for `p:&uniq 'r pool<T>`, so pool ops are callable repeatedly without moving `p`.) [FR *w reborrowing.]

== OWN-9 (addend) ==
The shared-borrow-read-only and owned-value-unaliased facts hold for every writer-constructible (kernel) type. The sole exception is the gated interior-mutable type `cell<T>` (§14, CELL-2). Kernel writers cannot construct `cell<T>`, so the facts are UNIVERSAL over kernel code.

== NEW [OWN-14] Result reborrows (call-boundary borrow provenance) ==
If a call's return mode is `&'r T` or `&uniq 'r T` and the instantiated region `'r` is supplied by exactly one borrow-mode argument, the result binding is recorded as a live borrow of that argument's resolved place (OWN-6), shared or `&uniq` per the return mode, for its OWN-4 lifetime; the argument's borrow is suspended for the result's lifetime (OWN-6 reborrow). A returned `&'r T`/`&uniq 'r T` whose `'r` is supplied by zero or by more than one borrow-mode argument names no determinate reborrow source and is rejected [OWN-14/DIAG-1] (OWN-8; general multi-source returned borrows routed to the §5 ratification pass, T-A singleton-preserving). A returned borrow rooted in a fresh callee allocation region is a new owned value, not a reborrow. Consequence (pool soundness): `pool_at`/`pool_at_uniq` reborrow the pool at `'r`, so while any element borrow is live, `pool_push`'s required `&uniq 'r pool<T>` conflicts under OWN-5 — realloc-on-grow cannot invalidate an outstanding element borrow.

== NEW [OWN-15] Region-generic aggregates ==
(1) Well-formedness (closes the 1E latent defect): the regions in scope inside a struct/enum are exactly its declared `region_params`; every REGIONID in any field/variant-payload type must be one of them. An out-of-scope field region is rejected [OWN-15/DIAG-1] (R4), replacing today's writable-but-never-constructible view field.
(2) Confinement (FR Def 3.21 Well-Formed-Type, lifted): a value of `S<..'a_i..>` is affine and counts, for the borrow judgments, as one live access per region argument `'a_i` of the resolved places its `'a_i`-fields borrow (`&uniq` iff a reachable field at `'a_i` is `&uniq`). Its liveness ends at the innermost `'a_i` block-end (OWN-4); it is storable/passable/returnable into region `'b` only if every `'a_i` outlives-or-equals `'b` (OWN-4); incomparable caller regions fail closed (OWN-3); OWN-5 and OWN-10 apply per `'a_i` unchanged. STOR-4 is the instance where the sole region argument bounds `arena` backing.
(3) Construct/project are region-substituted and exact-typed (no region variance in v0): `S<..'a_i..>(args)` requires each argument's type to equal the field's declared type under `'p_i:='a_i` (TYPE-5 exact match); `place.f` on `place:S<..'a_i..>` has the field's declared type under the same substitution. Each borrow/slice field registers its argument as held by the aggregate along that field path (OWN-6 holder resolution extended base-keyed → (base,field-path)-keyed; each field-borrow is one place, T-A singleton per field).
(4) Use-site argument order is declared order — `generics` (type/const) first, then `region_params` — the one convention shared with `fn` calls (FN-1). The keyword type-formers `slice`/`box`/`arena`/`array`/`pool`/`handle` are distinct grammar productions with intrinsic argument order and are not governed by this convention.

== NEW [OWN-16] replace/swap domain (T-A guard; table data, META-3) ==
`replace`/`swap` operate only on a region-free owned `T`: a `T` containing no borrow mode (`&`/`&uniq`) and mentioning no REGIONID (so not a borrow, `slice`, or region-instantiated aggregate). They write the destination place through `&uniq` while re-inhabiting it in the same operation, so OWN-1 whole-binding-death never fires (no dead interval, no uninit read) and single-ownership-out holds. The region-free/borrow-free restriction is forced by T-A (fr-reconciliation-m0.md): borrow-typed `T` would let `swap` exchange two borrows a flow-insensitive checker cannot track — the borrow-reassignment T-A bans.

== OP-1 (table, +8 rows; effect-row exactness inherits the §9 gate) ==
| `pool_new` | any T | `() -> own pool<T>` | allocates(heap) |
| `pool_push` | any T | `(&uniq 'r pool<T>, own T) -> own handle<T>` | writes('r), allocates(heap) |
| `pool_at` | any T | `(&'r pool<T>, handle<T>) -> &'r T` | reads('r), traps |
| `pool_at_uniq` | any T | `(&uniq 'r pool<T>, handle<T>) -> &uniq 'r T` | writes('r), traps |
| `pool_len` | any T | `(&'r pool<T>) -> own u64` | reads('r) |
| `handle_eq` | handle<T> | `(handle<T>, handle<T>) -> own Bool` | pure |
| `replace` | region-free owned T [OWN-16] | `(&uniq 'r T, own T) -> own T` | writes('r) |
| `swap` | region-free owned T [OWN-16] | `(&uniq 'r T, &uniq 's T) -> own unit` | writes('r, 's) |

== STOR-1 (addend) ==
`pool<T>` is heap-owned (box family): a single growable contiguous homogeneous buffer of owned `T`, single-owner and affine, dropped once at its binding's scope exit (STOR-3; no per-element free in the append-only v0 form). `handle<T>` is frame-resident copy: a non-owning index (u32 index; +u32 generation in the census-gated generational variant).

== STOR-3 (RC-ban note; no new construct) ==
UPHELD: shared ownership is a copied `handle<T>` into a single-owner `pool<T>` — zero refcount, no finalizer. RC remains re-admissible only with new cards; the deciding frontier census is registered (needs_experiment).

== §14 gated (writer-visible stub) ==
[CELL-1] Interior mutability is a gated capability TYPE `cell<T>` (`T` copy), NOT a writer mode. Cell semantics (copy-in/copy-out; no interior reference escapes). Opaque pre-approved signatures only: `cell_new(own T) -> own cell<T>` pure; `cell_get(&'r cell<T>) -> own T` reads('r); `cell_set(&'r cell<T>, own T) -> own unit` writes('r). `cell<T>` is absent from GRAM-3; a kernel program cannot name or author it (SCOPE-1); it appears only inside carded gated members (LEDGER-1).
[CELL-2] The sole invariant surrendered is OWN-9 shared-borrow-read-only, and only for the cell's own bytes: a shared borrow of `cell<T>` is neither read-only nor noalias. Type-visible, byte-localized carve-out lowered as LLVM `&UnsafeCell<T>` — storage emitted without `noalias`/`readonly`, loads/stores fenced by `alias.scope`/`noalias` metadata (verified-findings F3/ScopedNoAliasAA) so the exclusion does not leak to surrounding memory.
[CELL-3] `cell<T>` lacks `Shareable` (CAP-1): single-thread-confined; an atomic-cell (cross-thread shared-mutable) analog is a distinct future gated primitive.
RULING: interior mutability being a gated TYPE not a MODE, §5's writer mode set is COMPLETE at three {`own`, `&'r`, `&uniq 'r`}.

== LEX-1 (retire the deferral) ==
The DEFERRED "two-axis mode vocabulary (exclusivity × write-permission, frozen/shared-write)" is RETIRED for the shared-write axis: shared-write is the gated `cell<T>` TYPE (§14), never a mode. `frozen` (unique-read) remains a P0-only fourth-mode candidate, deferred until measured optimizer payoff earns it (R1).

== Worked substrate (non-normative; exact bytes are a democ task) ==
struct DNode { doc "CFG/DLL node: value + non-owning copy handles to neighbours."; value: own i64; prev: own Option<handle<DNode>>; next: own Option<handle<DNode>>; }
// Read neighbour handles via COEXISTING shared borrows (pool_at); do each field WRITE
// in its own single-element region (pool_at_uniq reborrows the whole pool, so two live
// uniq element borrows conflict — whole-pool exclusivity, OWN-7 posture). CFG = pool<Block>
// + pool<Instr> + per-block succ/idom handle fields; worklist = pool<handle<Block>>.

### Draft derivation-ledger rows
| OWN-14 | Result reborrows (call-boundary borrow provenance) | 🟡 existence-only | Existence T1-forced by counterexample (verified in checker.py: the `call` branch records no borrow for a return value, so pool_at's element borrow is unrelated to the pool and pool_push/realloc-under-live-borrow is a UAF that passes the checker — the same class as OWN-10). Form = FR *w reborrowing at the call boundary (fr-reconciliation-m0.md OWN-6/OBL-1), singleton-preserving (T-A). Selection: form minimality-selected (singleton-only; multi-source returned borrows rejected under OWN-8, over-rejection unmeasured, routed to §5 pass). | Not prototype-covered; requires call-result-borrow tracking (a real checker build). |
| OWN-15 | Region-generic aggregates (well-formedness, confinement, construct/project) | 🟡 existence-only | Existence evidence-forced: M4 self-host dogfood + W1 realistic-corpus need view-carrying aggregates; closes the 1E latent GRAM-3 self-contradiction (R4 latent-defect→loud-reject). Confinement form = FR Def 3.21 Well-Formed-Type, CONFIRMED letter-for-letter (memo OBL-0 pt 2) = OWN-4 lifted → inside the ratified calculus. Selection: mixed — confinement evidence/FR-selected; `[...]` grammar reuse and exact-match (no variance) minimality-selected. | Variance deferred (over-rejection on multi-region incomparable case, same class as OWN-3/10 7.2%, unmeasured); resolve-through-field is a checker build. |
| OP-1 pool/handle rows + STOR-1 | pool<T> single-owner indexed collection; handle<T> copy non-owning index | 🟡 existence-only | Evidence-selected: round2-memory-automation NAMES arena-index as the route AND its "well-typed slot-recycling UAF" hazard; append-only-no-recycle DERIVED as the exact T1 fix; in-distribution (rustc IndexVec, Cranelift entities, petgraph, slotmap). P0: single contiguous owned buffer preserves F001/F004 noalias, refcount-free. Selection: typed handle (not bare u64) evidence/R4-selected; append-only default T1-selected; growth couples to cluster 1C. | Generational per-element-free variant + RC-frontier census = needs_experiment. |
| OWN-16 | replace/swap on region-free borrow-free owned T | ✅ derived | R1 (OWN-1 whole-binding-death opened the hole; state machines/list splicing) + T1 (re-inhabit-in-same-op ⇒ no uninit window, no double-free, single-owner-out) + domain restriction T-A-FORCED (fr-reconciliation-m0.md: borrow-typed T = borrow reassignment the D1a bet bans) + META-3 (table data). Rust parity acceptable (mem::replace/swap): closes a hole, not a major decision. | Table-data domain; swap overlap already OWN-12. |
| CELL-1/2/3 (§14) + OWN-9 addend + LEX-1 | Interior mutability as gated cell<T> type, not a mode | ✅ derived | round2-memory-automation (no shared-mutable admitted before carding) + round3 unsafe-hatch (type-level escape envelope) + LEX-1 deferred two-axis vocab + W3 (surrendering OWN-9 must be gate-audited, not writer-emittable) → gated §14. Cell-not-RefCell (copy semantics, no interior refs) evidence-selected (misleading-comparisons #2; UnsafeCell = documented LLVM noalias opt-out, F3). SCOPE-1/LEDGER-1 placement. Retires LEX-1 shared-write deferral; §5 ratifies at three modes. | Concrete gated members (once<T>, memo<K,V>) await LEDGER-1 per-fact carding; atomic-cell deferred to concurrency layer. |
| OWN-6 addend | Reborrow-through-holder at call arguments | 🟡 existence-only | Existence forced: pool ops take `&uniq 'r pool`, but a uniq borrow is affine (OWN-1) with no reborrow form, so passing p twice consumes it — the snippets don't compile. Form = FR *w reborrowing (OWN-6 already cites it), additive, T-A singleton-preserving. | Requires a checker exemption (holder-borrow suspended, not conflicting) — a §5-wide build; confirm under OBL-1 extension. |

### Rust (R0) delta
(1) GRAPHS/SHARED-OWNERSHIP — the headline delta: Whitefoot forces ONE zero-overhead form, a single-owner contiguous `pool<T>` + copy `handle<T>`, so graphs lower to flat owned buffers with disjoint indexed access; F001/F004 noalias facts stay intact on every value except the one pool, and cache-dense flat storage is the vectorization substrate (pool_slice). Rust's idiomatic reach is `Rc<RefCell<Node>>`: atomic/non-atomic refcount traffic, RefCell runtime borrow-panics, and noalias loss at every shared node. Strictly more optimizer-visible facts, zero refcount, zero runtime borrow check. (2) INTERIOR MUTABILITY — `cell<T>` is a type-visible, byte-localized noalias carve-out the optimizer reads as a source fact (F3 scoped metadata), and it is the ONLY writer-unconstructible exception; so shared-borrow-read-only is UNIVERSAL over all kernel code. Rust's `UnsafeCell` is LLVM-invisible and forces rustc to conservatively drop noalias over a WIDER class (any type with a reachable interior-mutable byte), and Cell/RefCell/UnsafeCell are freely writer-usable — each silently defeats noalias wherever used. Whitefoot's noalias is TIGHTER and non-poisonable (W3: no dependency can silently defeat it). (3) SAFETY-VS-PARITY-PERF over Rust's own arena-index idiom: Rust arenas use bare `usize` (a stale index silently reads a recycled slot = silent corruption); Whitefoot's append-only pool makes intra-pool slot recycling UNREPRESENTABLE (T1 rung-1), and the census-gated generational variant TRAPS on stale access (R4). (4) replace/swap: HONEST PARITY with Rust mem::replace/swap (both get noalias from &uniq/&mut, register-resident) — the op is unavoidable and its noalias floor is met; the cluster's delta lives in (1)-(3), not here. HONEST NON-CLAIM (per critics): intra-graph element-vs-element noalias is NOT better than Rust Vec[i]/Vec[j] (ordinary same-base GEP alias analysis); the pool win is buffer-vs-other-memory noalias + refcount/RefCell-free + vectorizable whole-pool iteration, not element-pair disambiguation.

### Experiment spec
Two A/Bs, both blocked on building the M3 AI-codegen harness (the standing meta-blocker).

EXPERIMENT A — W1 idiom selection & the pool's usability cost. Arms: LOW / MID / HIGH capability models given the spec (this delta included) + task "build a doubly-linked list, then a CFG with a worklist and a reachability pass." Metrics per tier: (i) idiom-selection rate — reaches for pool+handle vs tries pointers/Rc/globals; (ii) self-reference correctness — writes `Option<handle<DNode>>` fields and reads them via match-through-shared-borrow without consuming the node; (iii) whole-pool-exclusivity friction — count of insert/mutate-while-element-borrowed rejections and repair-loop iterations to green (this measures whether per-element-region verbosity is a convergence problem); (iv) reborrow-vs-move errors — passing `&uniq p` bare (move) vs reborrowing `&uniq 'r deref(p)`; (v) replace-vs-move-then-reinit selection. Pre-committed read: if MID-tier (iii) repair loops or (ii) errors are high, add a disjoint-multi-borrow op (e.g. pool_split_uniq) and/or a sugar-free reborrow convention BEFORE FORM-1 freeze. Leading candidate: pool+handle (evidence-strong, in training distribution); W1 cost unmeasured.

EXPERIMENT B — RC-ban frontier census (STOR-3 "new cards" obligation). Measure, in performance-critical Rust AND the M4 compiler's own inventory (CFG, dom-tree, use-def chains, worklist, string-interner, type-table), the fraction of shared-ownership sites an append-only region/binding-confined pool CANNOT serve: sites needing per-element reclamation before the pool's scope ends, or lifetimes nesting in no single owner. Pre-committed thresholds: residue ~empty → append-only pool is complete, generational variant not admitted, gated Rc never admitted; residue moderate but region-confinable → admit the generational per-element-free pool variant (u32/u32, trap on generation exhaustion, census-gated opt-in); residue large + un-confinable → admit a GATED shared-ownership capability (not a kernel mode) with new STOR-3 cards. Leading candidate: append-only pool covers it (rustc TypedArena, Cranelift, GCC obstacks all region-confine IR).

(Also register: region-generic-aggregate over-rejection rate — TYPE-5 exact-match no-variance + incomparable-caller-region fail-closed — measured on the M4 corpus against the OWN-3/10 7.2% baseline, to decide whether additive region covariance earns its place.)

### Remaining owner rulings (5)
1. STRUCTURAL COPY (recommended ruling, FORM-1-urgent-if-reversed): do NOT adopt in v0. Keep FR Def 3.6 copy classification (primitives + shared borrows + now handle<T>); read nullable `Option<handle<T>>` neighbours via match-through-shared-borrow. This stays FR-consistent, keeps R3 one-form, and avoids the form1-breaking flip of every all-copy struct from affine→copy. Owner may revisit with Experiment-A data; reversing it later re-canonicalizes whether `move` is written.
2. POOL GROWTH vs cluster 1C: the growable backing buffer (pool_push realloc) is a cluster-1C (runtime-sized allocation) primitive; the handle/reborrow/confinement/append-only design here is complete and independent of it. Ruling owed: growable pool sequenced AFTER 1C lands, vs a fixed-capacity (construction-sized, no-realloc, no-1C-dep) v0 fallback. Leading: growable, sequenced after 1C.
3. GENERAL MULTI-SOURCE RETURNED BORROWS: a user fn returning `&'r T` selected among ≥2 same-region args is rejected in v0 (OWN-14/OWN-8, T-A singleton-preserving). The general lval-set treatment is routed to the §5 ratification pass as a recorded conservatism; confirm the reject rate is acceptable before ratifying §5.
4. CHECKER/DEMOC BUILD (honest, not free): landing this cluster requires building the type-tracking layer checker.py lacks (Binding has no type; no struct/field/signature table; the call branch records no return-value borrow; `use` cannot yet distinguish copy from affine so OWN-1's bare-affine ban is unenforced) PLUS OWN-14 result-reborrow provenance, OWN-15 resolve-through-field, OWN-16 domain check, and the OWN-6 reborrow-through-holder exemption. All frontend-scale (no NLL; T-A intact because handles are copy values adding zero borrow-provenance), but a real unbuilt subsystem — the spec text must not say 'rides existing machinery / zero checker cost.' democ needs aggregate/heap infra + the 1C runtime + bounds/generational trap emission.
5. GENERATIONAL VARIANT admission is gated on Experiment B (do not make generational the default access cost — its per-access gen-load+compare+branch is a P0 tax append-only does not pay).