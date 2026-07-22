# Pre-finalization open questions (triaged dossier)

Status: DRAFT 2026-07-07. Owner-requested. Input: a section-by-section weak-area audit of
`spec/kernel-spec-v0.4.md` (8 parallel auditors, one per section cluster; ~70 findings, full
JSON archived with the workflow run `wf_a8e8d429-74d`). This file consolidates, de-duplicates,
and triages those findings for owner rulings **before §-by-§ ratification freezes forms under
FORM-1**.

**How to read.** Every item is one of four tiers by what it blocks, tagged with:
- **Resolution mode** — `owner-ruling` (a scope/judgment call only the owner makes) · `design-card`
  (add a construct + evidence card; mostly additive) · `experiment` (needs the M3 AI-codegen
  harness / a measurement) · `already-gated` (deferral is recorded; just don't ratify past it).
- **⏰ FORM-1-urgent** — a later change to this is a *breaking canonical-byte change*, so it must
  be settled before a program corpus exists. (FORM-1 = "exactly one legal byte sequence"; changing
  a spelling later re-canonicalizes every existing program.)
- **Category** — from the audit: `current-defect` (spec contradicts itself), `absent-capability`,
  `minimality-selected` (R3-provisional), `procedural-breach`, `card-debt`, `gated-pending`,
  `human-residue`, `coupled-gap`.

**Caveat.** Findings are auditor-generated from the spec text; the internal-contradiction ones
(Tier 0) are reproducible by construction, but each design-card/experiment recommendation is a
*candidate*, not a decided direction — the point of this dossier is to route them, not pre-decide.

---

## Executive summary — three headlines

1. **The spec is not yet internally consistent.** At least 10 places the spec contradicts itself or
   its own normative example EX-1 — including that **EX-1's arithmetic calls (`iadd.checked<i32>(...)`)
   cannot be produced by the stated grammar** (GRAM-5 `call := IDENT`, but `iadd.checked` lexes as one
   OPNAME token), and **`idiv.checked` has no error variant for the `INT_MIN/-1` overflow** (a T2 hole:
   UB, hidden trap, or silent corruption). These are correctness bugs in the *specification*, mostly
   cheap owner-rulings, and they gate any "green-CI ratification."

2. **The language cannot yet express most real programs.** The audit found large `absent-capability`
   gaps that are *not* tracked anywhere: **no integer bitwise/shift/bit-count ops** (`band` is
   Bool-only), **`array<T,N>` has no constructor** (a first-class type nothing can build), **no
   runtime-sized allocation or any dynamic collection** (no Vec/Map/String), **no graphs/cyclic/shared
   structures** (so the M4 self-hosting compiler cannot express its *own* CFG/dom-tree data), **no
   I/O**, and **no float math beyond +−×÷** (no `sqrt`/`fabs`/`fmin`/`fma`/`fneg`). Your hex instinct
   was the visible tip: hex literals are half of the missing *bit-manipulation* stack.

3. **~Half the surface is R3-provisional (minimality-selected, unmeasured), and §5 is being
   recommended for ratification on soundness alone.** The FR-reconciliation discharges *soundness*;
   it does not touch the *W1 writability* cost of OWN-1/3/8/11/13, which the R3 register still lists
   as unvalidated. Soundness-complete ≠ ratification-ready for a language whose only writer is a weak
   model.

**Bottom line:** finalization is correctly blocked. The cheapest, highest-value first move is Tier 0
(fix the spec's self-contradictions) + deciding the Tier 1 *scope* questions (which absent
capabilities are in v0 vs deliberately excluded), because those determine what the R3 experiments in
Tier 2 must even measure.

---

## Recommended ratification-blocking shortlist (the must-settle-before-freeze set)

| # | Item | Tier | Mode | ⏰ |
|---|---|---|---|---|
| 1 | EX-1 not grammar-derivable (OPNAME call gap) | 0 | owner-ruling | |
| 2 | `idiv.checked` INT_MIN/-1 has no error variant (T2 hole) | 0 | owner-ruling | ⏰ |
| 3 | Out-of-range literal (`256_u8`) accepted with undefined value (T2 hole) | 0 | owner-ruling | |
| 4 | `cvt` totality contradiction (TYPE-4 vs OP-1) + undefined widening/narrowing partition | 0 | owner-ruling | ⏰ |
| 5 | Integer bitwise + shifts + bit-count ops absent | 1 | design-card | |
| 6 | `array<T,N>` unconstructable; no runtime-sized allocation / collections | 1 | design-card | ⏰ |
| 7 | Structs/enums cannot hold borrow/slice fields (no region-params) | 1 | design-card | |
| 8 | Graphs / shared-ownership / interior-mutability inexpressible (blocks M4 dogfood) | 1 | design-card | |
| 9 | I/O story (and a sound IO effect token so EFF-3 can't reorder it) | 1 | owner-ruling | ⏰ |
| 10 | §5 W1 costs (OWN-1/3/8/11/13) unvalidated — don't ratify on soundness alone | 2 | owner-ruling | ⏰ |
| 11 | The conditional complex (no-if / Bool-enum / statement-match / helper idiom) A/B | 2 | experiment | ⏰ |
| 12 | Loop form + prefix-arithmetic surface A/Bs | 2 | experiment | ⏰ |
| 13 | DIAG-3 spec-CI is red right now | 0 | owner-ruling | |

---

## Tier 0 — Internal defects (the spec contradicts itself; fix now, mostly cheap)

These are not design questions; they are places where the written rules are inconsistent, incomplete,
or admit undefined values. Most are owner-rulings that should land as a v0.5 errata pass.

| ID | Defect | Rules | Why it's a defect | ⏰ |
|---|---|---|---|---|
| D1 | **OPNAME operations have no call production.** `iadd.wrap` lexes as one OPNAME token (FORM-3), but `call := IDENT targs? "(" … ")"` and `construct := TYPEID …` — no production consumes OPNAME. So EX-1's own bytes aren't grammar-derivable; democ works only because its parser special-cases `.`. | FORM-3, GRAM-5, OP-1 | Canonical example unparseable by the canonical grammar; a checker built from the grammar rejects EX-1. | |
| D2 | **FORM-2 mandates a space before `:`.** The "no space before" set is `{) > , ; .}` — `:` absent — so the default one-space rule forces `a : own i32`, but EX-1 writes `a: own i32`. Every param/field/let hits it. | FORM-2, EX-1 | Two legal spellings; the byte format contradicts the byte-exact exemplar. | ⏰ |
| D3 | **EX-1 passes a borrow where the op wants a value.** `let p: &'r i32 = &'r a;` then `iadd.checked<i32>(p, …)`, but the op signature is `(i32, i32)`. TYPE-4/TYPE-5 forbid implicit conversion, so it needs `deref(p)` — or an unstated auto-read-through-shared-borrow rule. Second EX-1 contradiction. | EX-1, OWN-1, TYPE-4/5 | Normative anchor doesn't typecheck, or depends on an unwritten coercion (META-2 violation). | ⏰ |
| D4 | **Out-of-range literals accepted with undefined value.** No rule constrains literal magnitude to its suffix type's range; `256_u8`, `2147483648_i32` parse and violate no stated rule → accepted, value undefined. | FORM-5, TYPE-1, SCOPE-2 | T2/W3 silent-corruption channel; also blocks the negative-MIN fix. | |
| D5 | **`cvt` totality contradicts its own table.** TYPE-4: `cvt` is total "where value-preserving," else `Result`. OP-1: "widening int/float pairs" = total. But `i64→f64` widens yet loses values >2^53. | TYPE-4, OP-1 | The strongest-derived §4 rule contradicts the op table; writer can't predict the return type. | ⏰ |
| D6 | **`cvt` widening/narrowing partition undefined.** Which Src×Dst pairs are total vs `Result`? `i32→u32` (same width, resign)? `f64→i32` rounding mode, NaN, out-of-range — all unstated. | OP-1, TYPE-4 | The sole conversion op is unusable at the boundary; re-imports LLVM fptosi UB. | ⏰ |
| D7 | **`idiv.checked`/`irem.checked` can't represent `INT_MIN/-1`.** Signed division has two failure modes; the error enum is `DivideByZero` only. The overflow case → hidden trap (breaks ERR-4), wrong value (R4), or UB (T2). | OP-1, OP-2, PRE-1 | A "pure Result" op is non-total for its declared error type. | ⏰ |
| D8 | **`const IDENT: type` generic param is dead grammar.** GRAM-2 allows a const param `N`, but GRAM-3 `const := "[0-9]+"` (and CONST-1) means `array<i32, N>` doesn't parse and `N` has no value position. Generic-over-length is inexpressible. | CONST-1, GRAM-2/3, TYPE-2 | Grammar grants a construct the rest of the grammar gives no usable position. | ⏰ |
| D9 | **META-4 self-violation.** FORM-1 and META-1 both state the one-spelling fact with no cross-reference — the exact duplication META-4 forbids (a silent-drift channel). | META-4, META-1, FORM-1 | Live inconsistency in a CI-checked meta-rule. | |
| D10 | **DIAG-3 fails spec-CI right now.** The ledger row is keyed `| DIAG-3(v0.4.1: schemas delivered) |`, but `spec_ci.py:22` matches the literal `| DIAG-3 |` → `make spec` exits 1. Also masks any *future* orphaned rule. | DIAG-3 | Repo's own gate is red; no green-CI ratification possible; one-char fix. | |
| D11 | **String interior underspecified.** Escapes are only `\\ \" \n`; no `\t`/`\r`/`\0`/`\xNN`/`\u{}`, no statement of which raw bytes are legal inside quotes, no Unicode normalization → "one spelling per character" is unverifiable. | FORM-5, FORM-1 | Hash-invisible edit channel in a canonical-byte language. | ⏰ |

---

## Tier 1 — Absent capabilities (the language can't express real programs)

Grouped into coupled clusters. Each cluster is a *scope decision* (is this in v0?) plus, if yes, a
design-card. These are the "important in other languages" gaps you flagged.

### 1A · Integer bit manipulation — **coupled to hex literals** ⏰(hex)
Nothing in the op table does integer bit work: `band/bor/bxor/bnot` are **`Bool`-only**. Missing:
- `iand/ior/ixor/inot` (integer bitwise)
- `ishl/ishr` (+ signed=arithmetic vs unsigned=logical; shift-amount ≥ width policy) and `irotl/irotr`
- `popcount/clz/ctz/bswap` (single machine instructions; O(width)-loop emulation is slow and un-re-patternable)
- widening / high-half multiply (`u64×u64→u128`, `mulhi`)
- **hex/binary literals** (FORM-5 decimal-only) — masks are unreadable/error-prone in decimal, and *useless without the bit-ops above*.

**Why it matters (P0):** hashing, crypto, compression, bitsets, packing, SWAR — the language's own
stated systems/numeric domain — are entirely inexpressible. **W1:** a model emits `iand`/`shl`, is
rejected, and has no in-spec fallback. **Mode:** design-card (ops) + experiment (hex canonical
spelling, backlog 91f). Adding integer bitwise ops removes the *only* current use of the `band` name
collision as a trap.

### 1B · Arithmetic-ladder & float-math completeness ⏰(none, additive)
- **Saturating mode** `.sat` absent (only wrap/trap/checked) — SIMD `PADDS`/`UQADD` for DSP/audio/pixel/ML-quant.
- **`imin/imax/iabs`** absent (`iabs` has the INT_MIN hazard with no correct branchless form).
- **Float math absent:** `fneg`, `fabs`, `copysign`, `fmin`, `fmax`, `sqrt`, `floor/ceil/trunc/round`, `frem` — and the natural emulations are *IEEE-wrong* (`fsub(0.0,x)` ≠ `fneg` for ±0/NaN), so OP-3's "strict IEEE" promise is silently violated. IEEE-754 *mandates* sqrt and roundToIntegral.
- **`fma`** absent — and OP-3 `.strict` forbids contraction, so the accuracy-and-speed win is unrequestable without turning on unspecified fast-math.
- **`bitcast`/reinterpret** absent (f32↔u32) — needed for float hashing/serialization/bit-tricks; couples to 1A (can't get float bits into an integer to mask them).
- **Float comparisons asymmetric:** ints get all six; floats only `feq/flt/fle` (no `fne/fgt/fge`), and no ordered/unordered NaN predicate — an irregularity D2 calls the enemy.

**Mode:** design-card (all additive, each carded to its LLVM intrinsic). Verify the IEEE semantics as
table data (minNum/maxNum, sign-bit, NaN handling).

### 1C · Data: arrays, runtime-sized allocation, collections — **one coupled stack** ⏰
- **`array<T,N>` has no constructor** — a first-class type (TYPE-2, with `len`/`index`/`slice_of`) that *nothing can build*: `construct` needs a TYPEID, FORM-5 has no array literal, the op table has no `array_new`. No legal RHS for `let a: own array<i32,4> = …`.
- **No runtime-sized allocation** — `array<T,N>` needs literal `N` (CONST-1); `box_new`/`arena_new` wrap one `T`. No length-carrying heap/arena buffer → no vector/matrix of runtime size, i.e. no flat SIMD-friendly data.
- **No dynamic collections** — no Vec/HashMap/Set/String; prelude is 6 tiny enums; STOR-1 declares the storage-class set *closed*, so even a gated library has no primitive to wrap.
- **No named/aggregate constants** — no `const` item (item grammar is fn/struct/enum/contract/conform); constant tables (CRC/trig/masks) inexpressible; scalars must be nullary fns.

**Why it matters (P0):** flat runtime-sized arrays are *the* substrate the whole vectorization thesis
rests on; the M4 compiler needs growable worklists/symbol-tables. This is the single largest
expressiveness gap and it is **nowhere gated/deferred** — FORM-1 would freeze the hole. **Mode:**
design-card + owner scope-ruling (library vs builtin vs out-of-v0), coupled to the deferred const
sublanguage (D8/CONST-1).

### 1D · Memory shapes: graphs, sharing, interior mutability ⏰(none)
- **No cyclic/shared/back-referencing structures** — single-owner + affine + no-RC + no-globals + structs-can't-borrow ⇒ no CFG, dom-tree, doubly-linked list, parent pointers, DAG. **The M4 self-hosting compiler cannot express its own data structures**, which also means the W1 validation corpus can't contain realistic systems programs.
- **No shared ownership (Rc/Arc-analog)** — STOR-3 bans RC "re-admissible only with new cards"; the deciding frontier-sizing census never ran. Interning tables, shared config, hash-consing inexpressible.
- **No interior mutability** — LEX-1 defers the two-axis mode vocab; shared borrows are strictly read-only; no UnsafeCell-analog. Memoization, lazy-init, back-pointer updates, shared counters inexpressible.
- **No take-and-replace/swap** — OWN-1 whole-binding-death makes `mem::replace/swap/take` unwritable (state machines, list splicing).

**Mode:** design-card + owner-ruling. Card the arena-index-pool pattern and/or a gated shared-ref /
interior-mutability capability type — this is a named blocking sub-decision of the gated family.

### 1E · Structs can't borrow (no region-generic data types) ⏰(none) — **blocking + latent defect**
`struct_decl`/`enum_decl` carry `generics?` but **not `region_params?`**, and `gparam` has no
REGIONID production. Yet field types may be `slice<'r,T>`/`arena<'r,T>` (GRAM-3) — naming a region the
type can never bind. So **any view-carrying aggregate is grammatically writable but never
constructible** (a latent defect), and zero-copy parsers, iterators-as-structs, cursors, and
string-slices are impossible. **Mode:** design-card (add region params to struct/enum, or remove
region-carrying field types).

### 1F · I/O and observability ⏰(effect token)
No IO/syscall/clock/random op; `main` capped at `allocates(heap), traps`; a kernel program's only
observable effect is a trap report. All IO must cross the (unspecified) FFI wall — but **there's no
sound effect row for it**: labeling observable IO `pure` lets EFF-3 dedup/reorder it (a correctness
hazard), and no effect token forbids that. **Mode:** owner-ruling (IO = gated library vs builtin
effectful ops) + at minimum add an opaque `io`/foreign effect that blocks EFF-3 reorder/dedup.

### 1G · Error conversion across call chains ⏰
ERR-3 `try` is **same-E only** ("no conversions, TYPE-4"). A function calling libraries that return
`Result<_,IoError>` and `Result<_,ParseError>` and itself returning `Result<_,AppError>` can't `try`
either — it must hand-write the match+rewrap that ERR-3's own R4 rationale exists to eliminate. The
shift-left mechanism evaporates at the *most common* boundary. No From-law, no `try`-with-map, no
error-union. Also: `Err` carries no runtime origin context (only artifact-level). **Mode:**
design-card (one canonical conversion form). Tracked (ledger ERR-3 "deferred with delta").

### 1H · Generic numeric code ⏰
Literal suffix must be a concrete type — `0_T`/`1_T` for a generic `T` is undefined; OP-1 rows are
over concrete primitives, not a generic `T`; `gparam` bounds offer no numeric contract. So a generic
`sum<T>`/dot-product/min-max/accumulate can't form its identity or call `iadd` on `T`. Generics serve
almost no numeric use (an R2 "cut that harms codegen"). Also **no generic conformance / associated
types / multi-bounds / region-generic contracts** (`conform` has no generics; `gparam` is single-bound).
**Mode:** owner-ruling (do suffixes name params?) + design-card (numeric contract bound / associated
types).

---

## Tier 2 — R3-provisional surface forms (minimality-selected; need evidence before FORM-1 freezes)

These *exist* for good reason but their *specific form* was chosen for minimality, not measured under
weak writers. All ⏰ unless noted. Most need the M3 AI-codegen harness (which does not exist yet — its
absence is itself the meta-blocker, backlog "constitution_audit: build the AI-codegen validation
harness").

| ID | Provisional form | Rules | The tension | Mode |
|---|---|---|---|---|
| P1 | **Loop form** = bare `loop`+`break` only | GRAM-4/6 | R3-register anchor; hides A005 trip-count/independence/reduction facts the optimizer wants (P0); hand-rolled counter+guard is off-by-one-prone (W1). | experiment |
| P2 | **No `if`; match-on-Bool** (+ Bool-as-enum + no-bool-literals) | GRAM-6, PRE-1, FORM-5 | Never debated; every one-sided guard spells a dead `False()=>{}` arm; the whole complex flips together if the A/B flips. | experiment |
| P3 | **Statement-only match + helper-fn idiom** (baked into EX-1) | GRAM-7, EX-1 | Selected on the *literal* R3 disqualifier ("preserve one arm shape"); the record's own W1 signal favored the rejected value-arm form; top R2-reversal site. | experiment |
| P4 | **Prefix-call arithmetic, no operators** | OP-1, GRAM-6 | All AI-writer claims empirically untested; silent operand-order/associativity swaps (type-check → invisible) vs precedence-misbinding cut opposite ways. | experiment |
| P5 | **Prefix `deref(p)`/`index<T>(p,i)` places** | GRAM-5 | "Thinnest register item"; nested access `deref(index<T>(deref(p),i)).f` inverts natural reading order; paren/arg-order errors unmeasured. | experiment |
| P6 | **Positional-only construct (no named fields)** | GRAM-5, GRAM-2 | Two same-typed fields transposed type-check → *silent corruption* (R4's forbidden mode); not even on the register. | design-card |
| P7 | **TYPE-5 interior-annotation mandate** | TYPE-5 | **Procedural breach** — shipped normative while round-2 verdict = needs_evidence; the redundancy-independence experiment never ran. | experiment |
| P8 | **TYPE-6 no-shadowing** | TYPE-6 | "Cheapest" provenance; but M0 found *FR also bans shadowing* — re-ground on that formal-calculus lever instead of leaving it minimality-selected. (not ⏰) | design-card |
| P9 | **§5 W1 costs**: OWN-3/8/11 reject-when-unsure (5.5–7.2% over-rejection, no accepted threshold); OWN-1 whole-binding-death (stricter than FR); OWN-13 unwritten binder modes (the kernel's *only* unstated fact); OWN-10 is the sole measured over-rejection source with a shelved fix | OWN-1/3/8/10/11/13 | **§5 is recommended for ratification on soundness alone**; FR reconciliation doesn't touch these W1 costs. Don't conflate soundness-complete with ratification-ready. | owner-ruling + experiment |
| P10 | **STOR-3 compiler-derived-only drops, reverse-decl order** | STOR-3 | The deciding round-2 remand (writer-written free vs surfaced drops; lifetime-intent bug detection) never ran; reverse order is an untested convention. | experiment |
| P11 | **FN-3 contracts** (procedural breach, verdict=needs_evidence) · **FN-5 env-structs** (never A/B'd vs closures) · **FN-6 poly-recursion** (deliberately over-strong, not on register) | FN-3/5/6 | Abstraction layer chosen without the interfaces back-fill / defunctionalization-drift experiments. | experiment |
| P12 | **FORM-1 reject-vs-canonicalize** | FORM-1 | Never tested; a gofmt-style canonicalizer also yields canonical bytes and spends fewer repair loops. (not ⏰) | experiment |
| P13 | **FORM-5 decimal-only literals** | FORM-5 | Couples to 1A; registered A/B (backlog 91f). | experiment |
| P14 | **Human-residue rules**: FORM-2 formatting, FORM-4 no-comments/doc (ledger's *nearest-to-underived*), free-text trap/check message | FORM-2/4, SCOPE-4, OP-5, DIAG-3 | Encode info already carried structurally / prose in a language humans don't read; need an R1 card or a recorded R5 exception. (mixed ⏰) | design-card |
| P15 | **Control-flow ergonomics**: no `continue`, no integer/literal match arms or default (dense switch → comparison ladder, lost jump-table), no short-circuit `&&`/`||` (eager `band` traps on `i<len AND arr[i]`), `break` carries no value | GRAM-4/6, ERR-2, OP-1 | Each forces deep nested-match idioms; the short-circuit one is a *silent W1 trap* (the obvious `band` translation traps at runtime). | design-card |

Also here: **TYPE-1 inventory cuts** (no i128/u128, no usize/isize, no f16/bf16, no widening multiply, index-type unstated) — experiment (index-width is FORM-1-baked); **FN-4 LAWNAME closed to 3 laws**
(no distributivity/monotonicity/order — caps the "checked laws" R0 delta) — experiment; **META-2/3/4
regularity invariants** W1-asserted, never measured (regularity-vs-size ablation registered, unrun) —
experiment.

---

## Tier 3 — Gated / pending sections & card-debt (deferral is recorded; don't ratify past it)

These are already tracked as deferred; listed so the ratification gate is explicit.

| ID | Item | Rules | Status |
|---|---|---|---|
| G1 | **§9 effects gated** — 4-effect vocab never evidence-selected; **no effect polymorphism** (generics over contract methods must over-declare, defeating exact rows — the R0 delta lost at every abstraction boundary); no IO effect; concurrent-trap semantics unspecified; strict trap-order = vectorization cliff (no batch-replay rule) | EFF-1..4 | gated on exemplar carding (Cyclone/MLKit/Koka/DPJ) |
| G2 | **Concurrency/memory-model layer** — CAP-1 reserves 2 names, nothing else; no threads/atomics/fences/channels/parallel-reduction. **P0 forfeits multicore/SIMD-across-cores**; T1 "no data races" is trivially true only because concurrency is unrepresentable; memory model (UB vs bounded vs type-level) undecided | CAP-1 | research_needed |
| G3 | **§14 FFI family** — members unspecified; ffi-attenuation research_needed; FFI-IN/callbacks deferred (D4); the wall carries all boundary load and discharges T2's clause (b) | GATE-1, LEDGER-1 | gated; ffi_abi_runtime sources missing |
| G4 | **ir-strategy** — highest-priority open round-1 topic; DIAG-2 artifact format + backend-neutrality undecided; whether source facts survive lowering (the whole payoff) unvalidated end-to-end | DIAG-2/3 | research_needed |
| G5 | **Numeric fast-math** — OP-3 leaves all approximation/reassociation/contraction modes OPEN; strict is admissible-by-elimination, unpriced; "do not choose defaults until sources gathered" | OP-3 | gated (missing_primary_source) |
| G6 | **Arrays/vectorization facts** — minimal `array<T,N>`/`slice` carry no stride/shape/affine/SoA/independence facts; OWN-7 always-overlap slices + non-literal-index pessimism block the flagship vectorization | TYPE-2, OWN-7, OP-4 | prototype_needed |
| G7 | **Const-expression sublanguage** — deferred; ban-without-replacement grows the Go-`go:generate` external-templating risk (R2) | CONST-1 | deferred (time-sensitive) |
| G8 | **Closed-world compile-loop latency** — PROG-1 whole-program re-check + FN-2 re-instantiation, unbudgeted at AI edit scale; **the write→check→fix loop *is* the product** | PROG-1, FN-2 | blocking obligation, unrun |
| G9 | **Scope exclusions to record** — no separate compilation / dynamic loading / reflection / JIT / plugins / hot-reload; no shared versioned stdlib; fault isolation (Result-or-whole-process-abort only). Derived cuts, but the *excluded program classes* aren't recorded as explicit SCOPE limits | PROG-1, ERR-1, SCOPE-4 | owner-ruling (record the exclusions) |
| G10 | **Card-debt** — N002 only PARTIALLY_VERIFIED (grounds `.wrap`); checked-mode lowering uncarded; arena/region family has ZERO exemplar cards (FR doesn't cover arenas, so §5/§6 "reconciled" overstates); FN-2 anti-drift experiment unrun; ERR-2 edit-list repairability uncarded; D0a gate-efficacy revisit unscheduled | OP-1, STOR-1/2/4, FN-2, ERR-2, GATE-1 | design-card / experiment |

---

## Coupled-gap map (decide these *together*, or you'll re-canonicalize twice)

- **Bit stack:** integer bitwise ↔ shifts ↔ bit-intrinsics ↔ **hex/binary literals** ↔ bitcast. (1A + 1B + P13)
- **Data stack:** `array<T,N>` type ↔ array constructor ↔ runtime-sized alloc ↔ collections ↔ const sublanguage (`const N`) ↔ named-const items. (1C + D8 + G7)
- **Conditional complex:** no-`if` ↔ Bool-as-enum ↔ no-boolean-literals ↔ statement-only match ↔ helper-fn idiom ↔ EX-1 bytes. (P2 + P3)
- **View stack:** structs-can't-borrow ↔ region-params-on-structs ↔ zero-copy parsers / iterators-as-structs / cursors. (1E)
- **Graph stack:** graphs ↔ shared-ownership (RC) ↔ interior mutability ↔ arena-index-pool ↔ take/replace — all needed for the M4 compiler dogfood. (1D)
- **EX-1 integrity:** OPNAME-call-gap ↔ colon-spacing ↔ borrow-as-value ↔ helper-idiom baking — EX-1 needs a full re-cut once these settle. (D1 + D2 + D3 + P3)
- **Effect stack:** effect vocabulary ↔ effect polymorphism ↔ IO effect ↔ concurrent-trap ↔ trap-order vectorization. (G1)

---

## Resolution-mode rollup (what each item *needs from whom*)

- **Owner-rulings (scope/judgment; you decide, cheap):** all Tier 0 defects; the v0-scope calls in 1C/1D/1F/1H (library vs builtin vs out-of-v0); G9 scope exclusions; §5-ratify-on-soundness (P9); i128/index-width; fault-isolation policy.
- **Design-cards (additive + evidence; mostly mechanical):** the entire op-table completion (1A/1B — bitwise, shifts, bit-intrinsics, sat, min/max/abs, float math, fma, bitcast); array/collection primitives (1C); region-generic structs (1E); error conversion (1G); control-flow ops (P15); LAWNAME expansion.
- **Experiments (need the M3 harness — build it first):** every P-item surface A/B (loop, conditional, arithmetic, places, positional, shadowing, interior-annotation, env-structs, drops, reject-vs-canonicalize, decimal-vs-hex); §5 W1 costs; regularity-vs-size ablation. **Meta-blocker:** the AI-codegen validation harness that all these presuppose *does not exist yet*.
- **Already-gated (just sequence ratification behind them):** G1–G8 + G10 card-debt.

---

## Suggested sequence (cheapest-first, respecting FORM-1 urgency)

1. **v0.5 errata pass** — fix all Tier 0 defects (D1–D11). Mostly owner-rulings + text edits; makes
   the spec internally consistent and turns spec-CI green (D10). Add a `spec_ci` check that re-derives
   EX-1's bytes from FORM-2/FORM-3/GRAM (would have caught D1/D2/D3).
2. **Scope rulings** — decide, per Tier 1 cluster, *in v0 vs deliberately-excluded-and-recorded*: bit
   ops, data/collections, views, graphs/sharing, I/O, error conversion, generic numerics. This defines
   what "v0 the language" even is and what the experiments must measure. Record exclusions as explicit
   SCOPE limits (G9) so the trade is auditable.
3. **Op-table + data design-cards** — land the additive, evidence-clear ones (1A/1B/1C/1E/1G/P15).
   These are mostly "add a row carded to an LLVM intrinsic" and are FORM-1-additive (safe to do before
   the harness).
4. **Build the M3 AI-codegen harness** — the standing meta-blocker for every Tier-2 experiment. Until
   it exists, no R3-provisional form can ratify by evidence.
5. **Run the FORM-1-urgent surface experiments first** (P2/P3/P1/P4 conditional+loop+arithmetic, P9
   §5 W1 costs), because those freeze canonical bytes; defer the non-⏰ ones (P8/P12) if needed.
6. **Discharge the gated sections** (G1–G8) on their own tracks; keep §5 ratification behind its W1
   validation (P9), not soundness alone.

---

*Provenance: workflow `wf_a8e8d429-74d`, 8-cluster spec audit, 2026-07-07. Full per-finding JSON
(with `why_it_matters`, exact quotes, and `tracking` for each) is in the run transcript. This dossier
consolidates ~70 findings into ~50 triaged items; nothing was dropped silently — minor/duplicate
findings were merged into their cluster.*
