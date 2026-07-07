# Kernel Specification v0.3

Status: DRAFT v0.4.1 (2026-07-07). DIAG-3 field schemas added (closes the audit-flagged R4-load-bearing deferral). GRAM-4 scrutinee widened to expr, resolving the GRAM-4/EX-1 contradiction found by M1 construction (bind-then-match rejected: per R3/W1 it taxes the sole conditional idiom with a mechanical temporary at every use); OWN-13 gains the owned-temporary clause. v0.3.1 added ERR-3 propagation (closed the R4-load-bearing deferral). v0.3 was the ledger-fix revision of v0.2 (META-6 derivation discipline; OP-2 ineg.wrap added with corrected rationale scope; FORM-6/FN-6/FN-7 derivation rationales recorded). v0.2 was the lexicon revision of v0.1 (owner rulings: borrow-mode rename mut->uniq; lexicon policy LEX-1). v0.1 was the revision of v0 under the round-1 spec critique (63 findings, 37 missing rules; all blocking and major findings addressed or explicitly deferred with recorded deltas). Section 5 (ownership) is PROVISIONAL: its rules must be reconciled against a formalized minimal calculus (Featherweight Rust / Oxide / Austral class) before ratification, per the D1a gate. Section 9 (effects) is gated on region/effect exemplar carding before ratification.

Rule IDs are stable; diagnostics cite rule IDs. Sections marked DEFERRED record obligations with spec deltas per META-5, not normative content.

R3-PROVISIONAL REGISTER (constitution audit 2026-07-05; these forms were minimality-selected, not evidence-selected, and require validation before ratification — see decision-gates.md): loop form (GRAM-5/6), match-only conditionals and no-if (GRAM-6/PRE-1), statement-only match (GRAM-7), prefix arithmetic surface (OP-1/GRAM-6), interior annotation mandate (TYPE-5 — round-2 verdict still needs_evidence), no-shadowing (TYPE-6), env-struct closures replacement (FN-5), contracts/conform as interfaces replacement (FN-3 — round-2 verdict still needs_evidence), byte-format choices and reject-vs-canonicalize (FORM-1/2), no-comments (FORM-4), decimal-only literals (FORM-5), checker completeness levers (OWN-3/8/11 — rejection-rate unmeasured), deref/index prefix places (GRAM-5).

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel plus the writer-visible stubs of the gated family (§14). The gated family's members (unsafe regions, FFI extern frames, trusted primitive imports) are not writable by the steady-state writer; a kernel program contains no gated constructs.

[SCOPE-2] A program is accepted iff it parses under the canonical grammar, satisfies every rule in this document, and every unproven D1-critical checkable fact (bounds; alias-disjointness where a check form exists) carries a runtime check. There is no writer-emittable third state: nothing writer-stated is trusted unchecked. The sole trusted-assertion class is toolchain-gated ledger entries (§14), which the writer cannot author or edit.

[SCOPE-3] Accepted programs have no undefined behavior, conditional on: (a) the declared trusted computing base (compiler, checker, runtime, allocator, OS), and (b) when a program links gated FFI frames, ABI-well-behaved foreign code. This is the Layer-4 envelope statement; violations of (a)/(b) are outside the language's guarantee.

[SCOPE-4] Contract violation at runtime traps: the process emits a machine-readable trap report (§12) and aborts. There is no unwinding.

## 2. Canonical form

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting. Non-canonical input is a hard error; the toolchain never auto-formats. Unknown constructs are hard errors (conservative extension).

[FORM-2] Formatting, exhaustively: UTF-8; LF endings; file ends with exactly one LF; indentation exactly two spaces per `{` nesting level (match arms are one level inside `match`); exactly one space between adjacent tokens except: no space after `(` `<` or before `)` `>` `,` `;` `.`; one space after `,` and `:`; no space around `.` in places; no line wrapping (a statement is one line); declarations separated by exactly one blank line; no trailing whitespace.

[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*`; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.[a-z]+` (single token, e.g. `iadd.wrap`).

[FORM-4] There are no comments. Documentation is the `doc` field of declarations [GRAM-3]. Provenance lives in toolchain records.

[FORM-5] Literals, exhaustively: integers `[0-9]+_TYPE` (decimal only, mandatory suffix, e.g. `42_i32`); floats `[0-9]+\.[0-9]+_TYPE` (e.g. `1.5_f64`); `unit`; STRING `"..."` with escapes `\\ \" \n` only, one canonical escape per character (a string value has one spelling). STRING appears only in `doc` and `check` messages. There are no boolean literals: `Bool` is a prelude enum (§15).

[FORM-6] The token `unit` names the unit type in type position and the unit value in expression position; the grammar positions are disjoint productions, so resolution is production-local, not contextual. The lowercase spelling follows the primitive-type convention (TYPE-1: primitives are lowercase keywords, not TYPEIDs); the single-token value spelling is the R3 one-spelling choice for the type's sole inhabitant.

[LEX-1] Lexicon policy: surface names label checked invariants, stated in this document self-containedly. Names are never borrowed from backend IR vocabulary (e.g. `noalias`), which names lowering consequences, not source invariants; and a name is borrowed from another language's convention only where a divergence census shows the semantics genuinely match. Ruling of record: the exclusive borrow mode is `uniq` (uniqueness-type lineage), not `mut` (Rust divergence: exclusivity is the invariant; mutation is only its permission, and the name breaks under future interior-mutability capabilities). DEFERRED with recorded delta: the two-axis mode vocabulary (exclusivity x write-permission, adding frozen/exclusive-read and capability-gated shared-write).

## 3. Grammar

[GRAM-1] The grammar is deterministic and unambiguous (one parse per input; resolved with two-token lookahead where FIRST sets overlap). Every production maps 1:1 to one core-tree node kind; there is no desugaring.

[GRAM-2] Items:

```
program      := item*
item         := fn_decl | struct_decl | enum_decl | contract_decl | conform_decl
struct_decl  := "struct" TYPEID generics? "{" doc? field* "}"
field        := IDENT ":" type ";"
enum_decl    := "enum" TYPEID generics? "{" doc? variant* "}"
variant      := TYPEID "(" type_list? ")" ";"
fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"
                "->" rtype effects "{" doc? stmt* "}"
contract_decl:= "contract" TYPEID generics? "{" doc? fn_sig* law* "}"
fn_sig       := "fn" IDENT region_params? "(" param_list? ")" "->" rtype effects ";"
law          := "law" LAWNAME "(" IDENT ("," IDENT)* ")" ";"
conform_decl := "conform" type ":" TYPEID targs? "{" doc? fn_bind* "}"
fn_bind      := IDENT "=" IDENT ";"
doc          := "doc" STRING ";"
generics     := "<" gparam ("," gparam)* ">"
gparam       := TYPEID (":" TYPEID)? | "const" IDENT ":" type
region_params:= "[" REGIONID ("," REGIONID)* "]"
param_list   := param ("," param)*
param        := IDENT ":" mode type
type_list    := type ("," type)*
```

[GRAM-3] Types and modes:

```
type   := "i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|"f32"|"f64"|"unit"
        | TYPEID targs? | "array" "<" type "," const ">"
        | "slice" "<" REGIONID "," type ">" | "box" "<" type ">"
        | "arena" "<" REGIONID "," type ">"
rtype  := mode type
mode   := "own" | "&" REGIONID | "&uniq" REGIONID
targs  := "<" targ ("," targ)* ">"
targ   := type | REGIONID | const
const  := "[0-9]+"        # v0: integer literals only [CONST-1]
```

[GRAM-4] Statements:

```
stmt        := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt
             | break_stmt | region_stmt | check_stmt | match_stmt | try_stmt
try_stmt    := "let" IDENT ":" mode type "=" "try" expr ";"
let_stmt    := "let" IDENT ":" mode type "=" expr ";"
set_stmt    := "set" place "=" expr ";"
expr_stmt   := expr ";"
return_stmt := "return" expr ";"
loop_stmt   := "loop" LABEL "{" stmt* "}"
break_stmt  := "break" LABEL ";"
region_stmt := "region" REGIONID "{" stmt* "}"
check_stmt  := "check" expr "else" "trap" STRING ";"
match_stmt  := "match" expr "{" arm+ "}"
arm         := TYPEID "(" binder_list? ")" "=>" "{" stmt* "}"
binder_list := IDENT ("," IDENT)*
```

[GRAM-5] Expressions and places:

```
expr      := literal | "move" place | place | call | construct | borrow_expr
call      := IDENT targs? "(" arg_list? ")"
construct := TYPEID targs? "(" arg_list? ")"       # struct: fields in declared
                                                   # order; enum: variant payload
borrow_expr := "&" REGIONID place | "&uniq" REGIONID place
arg_list  := expr ("," expr)*
place     := pbase psuffix*
pbase     := IDENT | "deref" "(" place ")"
            | "index" "<" type ">" "(" place "," expr ")"
psuffix   := "." IDENT
```

[GRAM-6] There is no operator syntax, no precedence, no infix, no `if`, no `while`, no `for`. Conditional control is `match` on prelude `Bool` [PRE-1]; iteration is `loop` + `break`. `index` is a place (its sole home); bounds semantics are [OP-4].

[GRAM-7] `match` is a statement. The canonical conditional-initialization idiom is a helper function returning the value from `match` arms (worked example EX-1). This preserves one arm shape.

## 4. Types

[TYPE-1] Primitive types: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 unit`. (`Bool` is a prelude enum, §15, not a primitive.)

[TYPE-2] Composite types: `struct`, `enum`, `array<T, N>` (N a literal constant, [CONST-1]), `slice<'r, T>` (region-carrying view), `box<T>` (heap-owned unique), `arena<'r, T>` (region-bounded owned).

[TYPE-3] Nameability: every constructible type/mode/effect has a canonical, finite, writable name requiring no compiler execution.

[TYPE-4] There are no implicit conversions. Representation changes are explicit ops: `cvt<Src, Dst>(x)` is total where value-preserving for all inputs, and returns `Result<Dst, NarrowError>` otherwise (per the operation table, §7).

[TYPE-5] No inference across statements: every `let` states its full mode and type; call sites state all type/region/const arguments explicitly; argument types match declared parameter types exactly.

[TYPE-6] Name binding: declaration-before-use; a live name may not be shadowed or redeclared (one uniform rule for values, regions, labels); IDENT, REGIONID, and LABEL are disjoint namespaces; `break`'s LABEL must name a lexically enclosing loop.

[CONST-1] v0 constant expressions are integer literals only. The closed constant-expression sublanguage is DEFERRED (recorded delta: +1 section when added).

## 5. Ownership, regions, borrows (PROVISIONAL pending formal-calculus reconciliation)

[OWN-1] Every value has exactly one owner. Values are classified copy or affine: primitives (TYPE-1) and shared borrows copy on use; all other values (owned composites, `box`, `arena`, `slice` as `&uniq`, uniq borrows) are affine. An affine value is consumed by `move p` exactly once; a bare `place` expression of affine type is a hard error (write `move p`). After a move, the whole binding rooting `p` is dead (partial moves kill the whole binding); any later use, write, or `set` of a dead binding is an error — reinitialization requires a new `let`.

[OWN-2] Modes: `own` (owned), `&'r` (shared borrow in region `'r`), `&uniq 'r` (exclusive borrow in region `'r`). Modes are always written.

[OWN-3] Regions are lexical. `region 'r { ... }` introduces `'r`; `region_params` introduce caller-supplied regions. Region identifiers are unique within a function (parameters included). Outlives-or-equals is the total reflexive relation: `'a` outlives-or-equals `'b` iff `'a = 'b`, or `'a`'s block strictly encloses `'b`'s block, or `'a` is caller-supplied and `'b` is local. Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject).

[OWN-4] A borrow `&'a p` / `&uniq 'a p` is live exactly until the end of `'a`'s block (named-region liveness). It may be stored into a destination of declared region `'b`, passed to a parameter of region `'b`, or returned as `rtype` region `'b`, only if `'a` outlives-or-equals `'b`.

[OWN-5] Resolved-place exclusivity. While `&uniq 'a p` is live: no place overlapping resolved(`p`) may be read, written, moved, or borrowed, except reads/writes through that borrow's holder. While any `&'a p` is live: no place overlapping resolved(`p`) may be written, moved, or uniq-borrowed; reads are permitted. Content reached through any borrow may never be moved: `move` requires a place rooted at an own-mode binding.

[OWN-6] Holder and resolution. The holder of a borrow is the binding its `borrow_expr` initializes (a borrow not bound by `let` is a call-scoped temporary, live until the end of the enclosing statement). resolved(place) rewrites a place rooted at a holder binding to the borrowed place plus the appended suffix, recursively. All OWN-5/OWN-7 judgments use resolved places.

[OWN-7] Overlap: resolved `p` overlaps resolved `q` iff one is a prefix of the other. Two `index` places with the same resolved base overlap iff their indices are not both literals with unequal values. Two `slice` values over the same resolved root overlap conservatively.

[OWN-8] Reject-when-unsure: the checker rejects any program it cannot prove conformant. Rejection of a sound-but-unprovable program is not a defect; the diagnostic names the rule and a restructuring.

[OWN-9] Non-normative consequence for the optimizer: a live `&uniq` borrow's resolved place is unaliased by any other live access path; shared borrows are read-only for their duration; owned values are unaliased except by their own live shared borrows.

[OWN-10] Borrow-storage duration: `&'a p` is legal only if `p`'s storage outlives `'a`. For `p` rooted at an own-mode binding b: `'a` must be introduced within b's scope (never a caller-supplied region, for locals and own parameters alike). For `p` rooted at a borrow of region `'b`: `'b` must outlive-or-equals `'a`. For `p` rooted in `arena<'r, T>` content: `'r` must outlive-or-equals `'a`.

[OWN-11] Loops: inside `loop @l`, a `borrow_expr` may only name regions introduced inside `@l`'s body; bindings declared outside `@l` may not be moved inside it (copies exempt).

[OWN-12] Calls (OWN-CALL cluster): at a call, declared region parameters are substituted with the caller's region arguments, which must be live; argument borrows are live accesses of their resolved places for the duration of the call and are judged under OWN-5 (two `&uniq` arguments whose resolved places overlap are an error); the callee's effect row, instantiated at the actual regions, is checked against the caller's live borrows under OWN-5.

[OWN-13] Match ownership: a non-place expression scrutinee is an owned temporary (moved into the match). Matching a place of own mode moves it (the binding dies; binders receive `own` payloads); matching through `&'r` / `&uniq 'r` leaves the scrutinee live and binds payloads as `&'r` / `&uniq 'r` respectively. Binder modes are derived by this rule, stated once; they are not written.

## 6. Storage

[STOR-1] Storage class is a function of type, stated once: `box<T>` is heap-owned; `arena<'r, T>` is arena-owned, bounded by `'r`; every other owned value is frame-resident (inline in its owner or the stack frame). There is no per-binding storage annotation and no default clause. The reserved storage-contract field `foreign_shared` exists in the vocabulary but is legal only in programs containing gated FFI frames (§14); compiler-inferred demotion of an allocation to foreign-shared is a floor violation.

[STOR-2] Creation: `box_new<T>(v)` returns `own box<T>`; `arena_new<'r, T>(v)` returns `own arena<'r, T>`; both are ordinary calls in the operation table. Content access is through `deref`.

[STOR-3] Deallocation is compiler-derived and artifact-surfaced: every drop and arena release appears as an explicit operation in the elaborated artifact. Every control-flow edge leaving a region block (fallthrough, `break`, `return`) carries that region's releases and drops, in reverse declaration order. No finalizers; no reference counting.

[STOR-4] Arena confinement: a value of type `arena<'r, T>` may not be returned, stored into a field, or moved to a destination outside `'r`'s block; borrows of its content obey OWN-10 with source region `'r`.

## 7. Operations

[OP-1] Every computation is a call naming one operation from the operation table; one operation per (semantic operation × mode); nothing is overloaded. The table below is the normative inventory (columns: op, type domain, signature, effects).

| op | domain | signature | effects |
|---|---|---|---|
| `iadd.wrap` `isub.wrap` `imul.wrap` | all int T | `(T, T) -> own T` | pure |
| `iadd.trap` `isub.trap` `imul.trap` | all int T | `(T, T) -> own T` | traps |
| `iadd.checked` `isub.checked` `imul.checked` | all int T | `(T, T) -> own Result<T, Overflow>` | pure |
| `idiv.trap` `irem.trap` | all int T | `(T, T) -> own T` | traps |
| `idiv.checked` `irem.checked` | all int T | `(T, T) -> own Result<T, DivideByZero>` | pure |
| `ineg.wrap` | signed int T | `(T) -> own T` | pure |
| `ineg.trap` | signed int T | `(T) -> own T` | traps |
| `ineg.checked` | signed int T | `(T) -> own Result<T, Overflow>` | pure |
| `ieq` `ine` `ilt` `ile` `igt` `ige` | all int T | `(T, T) -> own Bool` | pure |
| `fadd.strict` `fsub.strict` `fmul.strict` `fdiv.strict` | f32 f64 | `(T, T) -> own T` | pure |
| `feq` `flt` `fle` | f32 f64 | `(T, T) -> own Bool` | pure |
| `band` `bor` `bxor` | Bool | `(Bool, Bool) -> own Bool` | pure |
| `bnot` | Bool | `(Bool) -> own Bool` | pure |
| `cvt` | widening int/float pairs | `(Src) -> own Dst` | pure |
| `cvt` | narrowing pairs | `(Src) -> own Result<Dst, NarrowError>` | pure |
| `len` | `slice<'r, T>`, `array<T, N>` | `-> own u64` | pure |
| `slice_of` | `array<T, N>` | `&'r place -> own slice<'r, T>` (a borrow of the whole array place) | pure |
| `box_new` | any T | `(own T) -> own box<T>` | allocates(heap) |
| `arena_new` | any T | `(own T) -> own arena<'r, T>` | allocates(arena 'r) |

[OP-2] There are no wrap modes for division/remainder because no sound modular semantics exists for divisor-zero; this is table data, not an exception clause. (Negation has a wrap mode: two's-complement wrapping negation is sound modular arithmetic — ledger fix 2026-07-07.)

[OP-3] v0 defines only `.strict` float modes (IEEE 754, no reassociation, no contraction); each node names its mode. Approximation/fast-math modes are an OPEN numeric-semantics question, not a decided layer.

[OP-4] `index<T>(p, i)` reads/writes are bounds-checked in all build modes when unproven; out-of-bounds traps [SCOPE-4]. "Proof" means deterministic-checker or verified-proof-artifact discharge; a solver may only promote performance-ledger facts and never licenses check elision.

[OP-5] `check e else trap "msg";` is a runtime check in all build modes, never elided. A passed check creates the checked fact on the dominated path (stated-and-checked channel); the fuller stated-and-checked vocabulary (loop invariants, ranges) is DEFERRED with its delta.

## 8. Functions, generics, contracts

[FN-1] Signatures state everything callers need: parameter modes/types, return mode/type, effect row, region parameters. Bodies are checked against signatures; callers rely only on signatures.

[FN-2] Generics are monomorphization-only; instantiation arguments are always explicit; expansion is compiler-side, pre-IR; instantiations are re-checked as concrete code.

[FN-3] Contracts: a `contract` declares fn signatures and laws; `conform T : C { member = fn; }` declares conformance, checked per member; at most one conformance per (type, contract).

[FN-4] Laws become optimizer-usable facts only via the stated-and-checked channel (static proof, runtime check under trap=abort, or verified proof artifact) or via gated-family ledger entries (§14). The law-test harness is non-normative prioritization for gate review; it never licenses optimizer use. LAWNAME is a closed table: `associative(f)`, `commutative(f)`, `identity(f, e)`.

[FN-5] No function values, no dynamic dispatch in the kernel. Behavior parameterization is generics over contract-conforming types (env-struct pattern); closed-set dispatch is `match`. Env-struct calls are guaranteed direct calls after monomorphization (never fn-pointer indirection). Typed operation tables and the mandated env-struct exactness diagnostics are DEFERRED constructs with recorded deltas.

[FN-6] Recursion is permitted. Polymorphic recursion is rejected by a syntactic rule: in any call cycle among generic functions, every call instantiates the callee at exactly the caller's own type parameters. This criterion is DELIBERATELY stronger than finiteness requires (it rejects some finite permutation cycles): predictable, locally explainable rejection per OWN-8's reject-and-restructure posture; the diagnostic must name the cycle and the restructuring. Rejection-rate measurement is a registered experiment.

[FN-7] Exactly one `fn main() -> unit` with effect row at most `allocates(heap), traps` must exist. There is no global state and no `'static` region in v0: ambient mutable globals would (a) erode the noalias fact base every function otherwise gets from parameter-only reachability (P0; carding backlog: GlobalsAA-class evidence), (b) create hidden inter-function channels invisible in signatures (W3, FN-1 signatures-as-trust-unit), and (c) pre-seed shared state for the future concurrency layer (T1).

## 9. Effects (gated on exemplar carding before ratification)

[EFF-1] Row grammar: `effects := "pure" | effect ("," effect)*` with `effect := "reads" "(" REGIONID+ ")" | "writes" "(" REGIONID+ ")" | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "traps"`, in exactly this canonical order (reads, writes, allocates, traps). `pure` is the unique spelling of the empty row. Frame residency (STOR-1) is not an allocation by definition.

[EFF-2] Exhibits is syntactic: a body exhibits `traps` iff it contains any `.trap` op, `check`, or bounds-checked `index` (even if later proven away); exhibits reads/writes/allocates per the operation table and borrow modes it uses. Rows are checked both ways against the syntactic definition: undeclared-but-exhibited and declared-but-unexhibited are both errors.

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

[DIAG-1] Every rejection cites exactly one rule ID, the node path in the canonical tree, and where applicable a mechanical fix or restructuring. Diagnostics are deterministic and byte-stable.

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

enum Option<T> { None(); Some(T); }

enum Result<T, E> { Ok(T); Err(E); }

enum Overflow { Overflow(); }

enum DivideByZero { DivideByZero(); }

enum NarrowError { NarrowError(); }
```

## 16. Worked example (normative bytes)

[EX-1] The following complete program is byte-exact canonical form:

```
enum Sign { Neg(); Zero(); Pos(); }

fn sign_of(x: own i32) -> own Sign pure {
  doc "Canonical conditional-initialization idiom: return from match arms.";
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
  doc "Borrow, region, checked arithmetic, match on Result.";
  let a: own i32 = 40_i32;
  region 'r {
    let p: &'r i32 = &'r a;
    let s: own Result<i32, Overflow> = iadd.checked<i32>(p, 2_i32);
    match s {
      Ok(v) => {
        check ieq<i32>(v, 42_i32) else trap "arithmetic drift";
      }
      Err(e) => {
        return unit;
      }
    }
  }
  return unit;
}
```

## 17. Spec meta-rules (CI-checked)

[META-1] One spelling per construct; productions map 1:1 to core-tree nodes.
[META-2] No context-dependent spellings or rule variants: no rule's meaning depends on surrounding context; defaulting rules do not exist.
[META-3] No rule carries an exception clause; conditional structure is expressed as total positive rules or table data.
[META-4] Every normative fact is stated once; other mentions are rule-ID cross-references.
[META-5] Every change to this artifact declares its spec delta (rules ±, tokens ±, spellings ±, exceptions ±) and its SELECTION GROUND (evidence-selected vs minimality-selected) in the decision gates; DEFERRED markers are tracked delta obligations.
[META-6] Every rule carries an entry in the derivation ledger (spec/derivation-ledger.md) tracing it to CONSTITUTION.md; a rule whose chain is refuted or orphaned (evidence card dies, constitutional premise amended) is automatically flagged for re-grounding; underived rules may not ratify.
