# Kernel Specification v0 (draft for calibration audit)

Status: DRAFT v0 (2026-07-02). Purpose: the first single normative artifact for the writer-facing kernel, drafted to (a) price spec mass for the calibration audit, (b) scope the checker-core prototype, (c) begin spec CI. It instantiates decided law: round-2 floors, round-3 safety envelope (writer-visible parts), D1a simplified ownership, round-4 regularity invariants. It is not yet ratified; every rule is subject to the decision gates.

Rule IDs are stable. Diagnostics cite rule IDs (round-4 law). Sections marked LAYER are outside the kernel and excluded from the kernel budget.

## 1. Scope and conformance

[SCOPE-1] This document defines the writer-facing kernel: everything an AI writer needs to produce accepted programs. The gated construct family (unsafe regions, FFI extern frames, trusted primitive imports) is NOT writable by the steady-state writer and is specified separately; a kernel program contains no gated constructs.

[SCOPE-2] A program is accepted if and only if it parses under the canonical grammar, satisfies every typing/ownership/effect rule, and every checkable-but-unproven D1-critical fact carries a runtime check (proven-else-checked). There is no third state: nothing writer-stated is trusted unchecked.

[SCOPE-3] Accepted programs have no undefined behavior. Contract-violation at runtime traps: the process emits a machine-readable trap report and aborts.

## 2. Canonical form

[FORM-1] There is exactly one spelling per semantic construct and one legal byte-level formatting. Non-canonical input is a hard error; the toolchain never auto-formats.

[FORM-2] Formatting: UTF-8; LF line endings; indentation is exactly two spaces per nesting depth; exactly one space separates tokens where separation is required; no trailing whitespace; declarations are separated by exactly one blank line.

[FORM-3] Identifiers: `[a-z][a-z0-9_]*` for values/functions/regions, `[A-Z][A-Za-z0-9]*` for types/contracts/variants. No other identifier forms exist.

[FORM-4] There are no comments in canonical source. Documentation is the `doc` field of declarations (§4). Provenance lives in toolchain records, not source.

[FORM-5] Literals: decimal integers with mandatory type suffix (`42_i32`); floats with mandatory suffix and decimal point (`1.5_f64`); `true`, `false`; `unit`. No other literal forms (no hex/octal/underscore grouping in v0).

## 3. Grammar

[GRAM-1] The grammar is context-free, LL(1), and isomorphic to the checked core tree: every production corresponds to one core-tree node kind; there is no desugaring.

[GRAM-2] Program:

```
program   := item*
item      := fn_decl | struct_decl | enum_decl | contract_decl
```

[GRAM-3] Declarations:

```
struct_decl   := "struct" TYPEID generics? "{" doc? field* "}"
field         := IDENT ":" type ";"
enum_decl     := "enum" TYPEID generics? "{" doc? variant* "}"
variant       := TYPEID "(" type_list? ")" ";"
fn_decl       := "fn" IDENT generics? "(" param_list? ")" "->" type
                 effects region_params? "{" doc? stmt* "}"
param         := IDENT ":" mode type
contract_decl := "contract" TYPEID "{" doc? fn_sig* law* "}"
doc           := "doc" STRING ";"
```

[GRAM-4] Statements and expressions:

```
stmt      := let_stmt | set_stmt | expr_stmt | return_stmt
           | loop_stmt | break_stmt | region_stmt | check_stmt
let_stmt  := "let" IDENT ":" mode type "=" expr ";"
set_stmt  := "set" place "=" expr ";"
return_stmt := "return" expr ";"
loop_stmt := "loop" LABEL "{" stmt* "}"
break_stmt := "break" LABEL ";"
region_stmt := "region" REGIONID "{" stmt* "}"
check_stmt := "check" expr "else" "trap" STRING ";"
expr      := literal | place | call | match_expr | borrow_expr
call      := path targs? "(" arg_list? ")"
match_expr := "match" expr "{" arm+ "}"
arm       := TYPEID "(" binder_list? ")" "=>" "{" stmt* "}"
borrow_expr := "&" REGIONID place | "&mut" REGIONID place
place     := IDENT | place "." IDENT | "index" targs "(" place "," expr ")"
```

[GRAM-5] There is no operator syntax, no precedence, no infix. All operations are call-shaped (§7). There is no `if`: `bool` is `enum Bool { True(); False(); }` in the prelude and is dispatched by `match`. There is no `while`/`for`: iteration is `loop` + `break`.

## 4. Types

[TYPE-1] Primitive types: `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 bool unit`.

[TYPE-2] Composite types: `struct` (nominal product), `enum` (nominal sum), `array<T, N>` (fixed length, N a const generic), `slice<T>` (view; always borrowed, never owned).

[TYPE-3] Every type has a canonical, finite, writable name requiring no compiler execution to produce (nameability guarantee).

[TYPE-4] There are no implicit conversions of any kind. Every representation change is an explicit call-shaped operation naming source and target type (`cvt<i32, i64>(x)`), and every such operation is value-preserving or returns `Result` (§7).

[TYPE-5] There is no type inference across statements. Every `let` states its full type and mode. Within a single expression, argument types must match declared parameter types exactly.

## 5. Ownership, regions, borrows (D1a simplified calculus)

[OWN-1] Every value has exactly one owner. Ownership is affine: a value is moved at most once; after a move the source binding is dead, and any later use of a dead binding is an error.

[OWN-2] Modes on bindings and parameters: `own T` (owned), `&r T` (shared borrow in region r), `&mut r T` (exclusive borrow in region r). Modes are always written; there is no default.

[OWN-3] Regions are lexical. `region 'a { ... }` introduces region `'a`; a function's `region_params` introduce caller-supplied regions. Region `'a` outlives `'b` iff `'a`'s block strictly encloses `'b`'s block; a caller-supplied region outlives every local region.

[OWN-4] A borrow `&'a p` or `&mut 'a p` names the region whose scope bounds the borrow's life. A borrow may not escape its region: it may not be stored into a value or returned unless the destination's declared region outlives-or-equals `'a`.

[OWN-5] Exclusivity: while an `&mut 'a p` borrow is live, no other borrow of `p` or of any overlapping place is live, and `p` itself may not be read, written, or moved except through that borrow. While any `&'a p` is live, `p` may not be mutably borrowed, written, or moved.

[OWN-6] Liveness is lexical in v0: a borrow is live from its creation to the end of the enclosing region block. (Flow-sensitive shortening is a permitted future refinement; programs rejected under lexical liveness must be restructured.)

[OWN-7] Place overlap: `p` overlaps `q` iff one is a prefix of the other in the place grammar. `index` places overlap conservatively unless indices are distinct constants.

[OWN-8] The checker rejects any program it cannot prove conformant with OWN-1..OWN-7. Rejection of a sound-but-unprovable program is not a defect; the diagnostic must name the rule and a restructuring.

[OWN-9] Consequences the optimizer may assume for accepted programs: an `&mut` borrow does not alias any other live access path; owned values are unaliased; shared borrows are read-only for their duration.

## 6. Storage and allocation

[STOR-1] Every allocation names its storage class explicitly: `stack` (function frame), `arena<'a>` (region-bounded arena), `heap` (owned unique allocation).

[STOR-2] `let` bindings of non-borrowed values are `stack` unless the initializer is an explicit allocation call (`alloc_arena<'a, T>(v)`, `alloc_heap<T>(v)`).

[STOR-3] Deallocation is compiler-derived from ownership and region ends, and is surfaced: every drop and arena release appears as an explicit operation in the elaborated artifact. There are no finalizers and no reference counting in the kernel.

[STOR-4] Arena allocations die with their region; heap allocations die when their owner dies. Escaping an arena value past its region is an OWN-4 violation.

## 7. Operations

[OP-1] Every computation is a call-shaped operation with explicit type arguments where the signature is generic. There is exactly one operation per (semantic operation × mode); nothing is overloaded.

[OP-2] Integer arithmetic names exactly one mode per node: `iadd.wrap<T>`, `iadd.trap<T>`, `iadd.checked<T>` (returns `Result<T, Overflow>`); likewise `isub`, `imul`, `idiv` (div: `.trap` and `.checked` only; divisor zero traps or errors — no wrap mode), `ineg`, comparisons `ieq/ilt/ile<T>` (mode-free, total).

[OP-3] Float arithmetic is strict IEEE 754 by default: `fadd.strict<T>` etc. Reassociation/FMA/approximation permissions are LAYER (scoped fast-math annotations), not kernel.

[OP-4] `index<T>(s, i)` is bounds-checked: unproven bounds carry a runtime check in all build modes; out-of-bounds traps [SCOPE-3]. The compiler may eliminate a check only by proof.

[OP-5] `check e else trap "msg";` is the writer-visible assertion: it is a runtime check in all build modes, never elided, and its trap report carries the message and rule context.

[OP-6] Boolean operations are `band/bor/bnot` (total, non-short-circuiting); conditional evaluation is expressed with `match`.

## 8. Functions, generics, contracts

[FN-1] A function signature states everything callers need: parameter modes and types, return type, effect row (§9), region parameters. Signatures are the unit of trust: bodies are checked against signatures; callers rely only on signatures.

[FN-2] Generics are monomorphization-only. Every call site supplies explicit type arguments; there is no inference and no bound solving beyond direct conformance lookup. Instantiation is compiler-side; the writer never expands templates.

[FN-3] Generic parameters may be constrained by contracts: `fn sum<T: Monoid>(...)`. A contract is a named bundle of function signatures and laws. Conformance is declared explicitly per (type, contract) and checked per member; there is at most one conformance per (type, contract).

[FN-4] Laws (`law associative(combine);`) are optimizer-usable facts; each law is either checked by the toolchain's law-test harness or discharged by the gated channel — never assumed from declaration alone.

[FN-5] There are no function values in the kernel: calls name a declared function. Parameterizing behavior is done with generics over contract-conforming types (env-struct pattern). Dynamic dispatch does not exist in the kernel; closed-set dispatch is `match` over enums.

[FN-6] Recursion is permitted; polymorphic recursion is not (instantiation closure must be finite).

## 9. Effects

[EFF-1] Every function declares an effect row chosen from: `pure`, `reads('r...)`, `writes('r...)`, `allocates(storage...)`, `traps`. The row is exact: a body exhibiting an undeclared effect is rejected; declaring an unexhibited effect is rejected (rows are checked both ways).

[EFF-2] `pure` functions read only their parameters, write nothing external, allocate nothing, and cannot trap. Pure calls with equal arguments are equal; the optimizer may deduplicate, reorder, and eliminate them.

[EFF-3] `traps` marks possible trap exits (checked ops with unproven facts, `check`, `.trap`-mode arithmetic). Trap is abort: no unwinding, no cleanup semantics, no observable intermediate state contract beyond the trap report.

## 10. Errors

[ERR-1] Recoverable errors are values: `Result<T, E>` and `Option<T>` are prelude enums dispatched with `match` like any enum. There are no exceptions, no unwinding, no panics-as-control-flow.

[ERR-2] Every `match` is exhaustive over declared variants; wildcard arms do not exist. Adding a variant is a source-breaking change surfaced as a site-enumerated edit list by the toolchain.

## 11. Programs, closed world

[PROG-1] A program is one closed compilation unit: every name is defined within it (plus the prelude). There is no separate compilation, no internal stable ABI, no dynamic loading, no reflection. The only boundary is the FFI wall, which is gated and not kernel-writable.

[PROG-2] Entry point: `fn main() -> unit` with effect row not exceeding `writes('static), allocates(heap), traps`.

## 12. Diagnostics contract (toolchain floor)

[DIAG-1] Every rejection cites exactly one rule ID from this document, the node path in the canonical tree, and where applicable a mechanical fix or restructuring suggestion. Diagnostics are deterministic and byte-stable for identical inputs.

[DIAG-2] Accepted programs elaborate to a canonical artifact in which every derived operation (drops, arena releases, monomorphized instantiations, retained runtime checks) is explicit. The artifact is the re-read source of record.

## 13. Spec meta-rules (CI-checked)

[META-1] One spelling per construct; grammar productions map 1:1 to core-tree nodes [GRAM-1].
[META-2] Zero context-dependent rules: every rule is locally decidable at its node given declared types in scope.
[META-3] No rule carries an exception clause.
[META-4] Every normative fact is stated once; other mentions cross-reference its rule ID.
[META-5] Every design change to this artifact declares its spec delta (rules added/removed, tokens, spellings, exceptions) in the decision gates.
