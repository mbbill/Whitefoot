# Port plan — compiler v0.59 → v0.60

Synthesis of the seven subsystem maps in this directory plus `corpus-plan.md`.
Spec read-only at `/private/tmp/whitefoot-spec-x1`. No `cargo` command was run.

**Verified** means read out of the tree or one of the two specification files at
the cited line. **Inferred** means concluded from what was read but not stated
anywhere. Every line figure is an estimate built from verified current counts.

---

## 0. Two blocking defects, found by reading, before any code moves

Both are in v0.60 itself, not in the compiler. Neither can be worked around by
the port; both must be ruled before the fixture corpus or the prelude is written.

**D1 — v0.60's own `[PRE-1]` prelude does not parse under v0.60's own `[GRAM-2]`.**
Verified: `gparam := TYPEID ":" (TYPEID | linearity_bound) | "const" IDENT ":" type
| fn_sig | pack_use` (`spec/kernel-spec.md:207`) makes a bound mandatory on a type
parameter. Verified: the new records write bare parameters —
`fn box_new<T>` (`:2189`), `fn place_back<W, T>` (`:2222`), `fn take_back<W, T>`
(`:2228`), `fn append<W, X>` (`:2243`), `fn grow<T>` (`:2254`), `fn swap<T>`
(`:2267`), `fn free_empty<W>` (`:2268`). This is not cosmetic: verified at
`compiler/src/prelude.rs:1` ("Ordinary PRE-1 declaration records, parsed by the
same grammar as source declarations") and at `prelude.rs:427`, where the records
are lexed, parsed, resolved and checked by the real pipeline. As written the
prelude fails at the parser.

**D2 — `[TYPE-9]` writes a capacity spelling the grammar cannot read.**
Verified `spec/kernel-spec.md:513`: `Array<T, N>`, `Slots<T, N>`, `Ring<T, N>`,
"whose capacity is the type constant N [CONST-1]". Verified `:563`:
`const := ("[0-9]+" | IDENT) (infix_op ("[0-9]+" | IDENT))?`, and `[FORM-3]`
(`:93`) fixes IDENT as `[a-z][a-z0-9_]*`. A capital `N` in a `targ` slot parses
as the `type` arm (TYPEID), never the `const` arm. Verified that the `[PRE-1]`
fence itself uses lowercase — `fn slots_from_array<T, const n: u64>(values: own
Array<T, n>)` (`:2214`) — so the parser is already right and `N` in TYPE-9 is
prose. **Recommendation: confirm `N` is a prose metavariable and leave the
grammar alone.** The alternative (admit a capitalized const spelling) amends
GRAM-3 and CONST-1 and collides with the TYPEID domain.

---

## 1. Totals

### Counting convention

Each file is assigned to exactly **one** subsystem, so these totals are
deduplicated across the seven maps. Six files are claimed by two or three maps
and are assigned as follows: `places.rs`, `model.rs`, `check.rs`,
`semantic/mod.rs` → ownership; `permission.rs`, `loop_permission.rs`,
`permission_ledger.rs` → parallel; `calls/user.rs`, the entailment tree,
`kernel.rs`, `goal.rs` → effects-calls; every `*/tests/*` file except the lexer
and syntax test modules → tests-tools; `tests/conformance/**` → corpus.

- **Deleted** — lines removed with the module or item that holds them.
- **Rewritten** — lines of *existing file surface* that the port changes. It is
  a surface figure, not an edited-line figure; see the caveat below.
- **New** — estimated net-new lines in modules that do not exist today.

| Subsystem | Deleted | Rewritten | New | Untouched | Now | After (est.) | Net |
|---|---:|---:|---:|---:|---:|---:|---:|
| syntax (lexer, grammar, parser) | 0 | 9,021 | 0 | 5,065 | 14,086 | 13,964 | −122 |
| resolution (+ prelude catalog) | 598 | 4,227 | 3 | 2,827 | 7,652 | 7,190 | −462 |
| semantic — ownership & model | 3,943 | 19,490 | 2,200 | 0 | 23,433 | 17,350 | −6,083 |
| semantic — effects, calls, entailment | 324 | 35,267 | 420 | 6,801 | 42,392 | 39,501 | −2,891 |
| semantic — parallel permission | 453 | 3,811 | 0 | 0 | 4,264 | 3,157 | −1,107 |
| lowering, backend, driver | 1,261 | 21,402 | 0 | 23,263 | 45,926 | 43,206 | −2,720 |
| tests, programs, libraries, CI | 980 | 73,736 | 400 | 18,341 | 93,057 | 86,081 | −6,976 |
| conformance corpus | 3,680 | 20,419 | 610 | 13,312 | 37,411 | 33,222 | −4,189 |
| **Total** | **11,239** | **187,373** | **3,633** | **69,609** | **268,221** | **243,671** | **−24,550** |

Corpus rows are derived: verified 36,015 `.wf` lines over 1,133 cases
(≈32 lines/case) against the corpus plan's 416 unchanged / 602 rewrite /
115 retire / 19 new, plus the verified 1,155-line manifest. Docs (≈2,300
rewritten, 374 deleted, 70 new) sit outside this table; see P14.

### Caveat on "Rewritten"

Three files inflate the effects-calls figure because the whole file is listed but
only named spans change: `entailment/flow.rs` (16,436 lines, ~1,200 in the named
kill spans), `check/generics.rs` (2,828 lines, two call sites),
`entailment/flow/sources.rs` (1,757 lines, one new fact source). Discounting
those three, effects-calls rewrites ≈**15,700** lines and the repository total
≈**168,000**. The test figures are whole-file counts; the tests map measured
2,631 lines of retired vocabulary across 209 files, which is the mechanical
half — the judgment half is re-deriving what each case is still evidence for.

### Where the work actually is

| | Share of rewritten lines |
|---|---:|
| Rust tests (semantic, backend, syntax, lowering, driver, `compiler/tests`) | 61,043 (33%) |
| Non-test semantic (`semantic/**`) | 58,568 (31%) |
| `.wf` corpus (conformance cases + `tests/programs` + `lib`) | 39,369 (21%) |
| Lowering, backend, driver non-test | 18,297 (10%) |
| Syntax, resolution, prelude | 10,096 (5%) |

---

## 2. Work packages

### The buildability fact

The tree **cannot** be kept buildable between P2 and P11. Verified reasons:

1. `Production` and `FixedTerminal` are *generated* from the spec bytes by
   `compiler/build.rs:66-68`. Changing the spec file regenerates both enums, and
   the syntax map counted **600+** references to `Production::` /
   `FixedTerminal::` / `TerminalPredicate::` / `NamePredicate::` outside
   `syntax/` and `lexer/` — 109 in `resolution/engine/roles.rs`, 73 in
   `semantic/check/types.rs`, 55 in `check/requires.rs`, 54 in `check/borrows.rs`.
2. `semantic/check/borrows.rs:83` and `check/control/results.rs:16` are
   **compile-time** `DIAG-1` rank assertions. They stop the build the instant
   `definition_rank` changes, so `semantic/mod.rs`'s rank table must be rebuilt
   before any other semantic module compiles — the port cannot be sequenced from
   the leaves.
3. Deleting `KernelOperationId` / `ResolvedTarget::Kernel` breaks 20 match arms
   in `lowering/builder/runs.rs:165-445` plus six semantic files at once.

So P1 is the only package that leaves the gate fully green, and P11 is the first
package after which `cargo build` succeeds. P3 is the useful exception: the
conformance stage is **pure Python** (`make conformance` runs
`tests/conformance/test_runner.py` and `runner.py coverage`, verified at
`Makefile:188-190`) and needs no compiler, so it can be landed and proved green
in the middle of the unbuildable stretch.

### The packages

| # | Package | Subsystems | Del | Rewrite | New | Gate after |
|---|---|---|---:|---:|---:|---|
| 1 | Pre-port consolidation, valid under v0.59 | sem-own, sem-eff, sem-par, syntax | 450 | 1,800 | 400 | **full `make check` green** |
| 2 | Spec bytes, grammar tables, lexer, parser | syntax | 0 | 9,021 | 0 | `spec-append-only`; build red |
| 3 | Conformance manifest and case corpus | corpus | 3,680 | 20,419 | 610 | **`conformance` green (no build)** |
| 4 | Resolution and the prelude catalog | resolution | 598 | 4,227 | 3 | none |
| 5 | Rule table, checked model, place relation | sem-own | 0 | 4,800 | 120 | none |
| 6 | References and the ownership core | sem-own | 3,943 | 8,947 | 1,700 | none |
| 7 | Effects, calls and entailment | sem-eff | 324 | 35,267 | 420 | none |
| 8 | Storage shapes and windows | sem-own | 0 | 2,046 | 500 | none |
| 9 | Parallel permission | sem-par | 453 | 3,811 | 0 | none |
| 10 | Lowering | low-be | 774 | 9,038 | 0 | none |
| 11 | Backend, driver, pinned sentences | low-be | 487 | 12,364 | 0 | **`static` green; `cargo build`+`clippy` pass** |
| 12 | Compiler test suites | tests | 980 | 54,786 | 400 | `_check-unit` |
| 13 | Program corpus, libraries, compute hosts | tests | 0–3,026 | 18,950 | 0 | `_check-corpus`, `_check-runtime`, `_check-libraries` |
| 14 | Docs and the design tree | docs | 374 | 2,300 | 70 | `spec-prose-integrity`, `design-lint`; **`make check` green** |

### Why each package sits where it does

**P1 — Pre-port consolidation (v0.59-valid).** The only package that both
advances the port and keeps the gate green, so it is the only one that can be
reviewed against a working compiler. Content: collapse the five near-identical
path types the effects map found (`CheckedStatePath` root+`Vec<u32>`,
`ResolvedPlace` ×2 with different roots, `PlaceTerm`, `ProjectedPlaceTerm`,
`CallDatumProjection`) into one step-path type; replace
`Checker::state_path`'s silent truncation at `check/borrows.rs:472-486` with a
fallible conversion; add a `const` assertion that `TerminalSet`'s index stays
under 128 (`syntax/terminal.rs:622` is a `u128` with 27 bits spare and no guard);
fix the two stale rule citations (`PROV-3` in `semantic/tests/slices.rs` and
`tests/programs/runs.rs`, `PAR-3` in `tests/programs/network.rs` — verified
defined in neither version). Every one of these is a soundness or churn risk the
port would otherwise carry into the dark; the truncation in particular becomes a
v0.60 under-approximation of a write, which makes EFF-5 admit an overlapping pair
it must refuse.

**P2 — Grammar first, because everything is generated from it.** Verified: the
parser is fully table-driven and hardcodes only `Production::{Program, Item,
FnSig, ContractBlock}` and `FixedTerminal::Semicolon`, all of which survive — so
the five deleted and five new productions cost *zero* hand-written parser code.
The cost is elsewhere: the fixed-terminal inventory drops 103→94 and, because
`heap_decl` is `GRAM-2`'s third production, `program`/`no_heap`/`;` take ordinals
0-2 and **every** ordinal shifts, so `terminal.rs` is regenerated rather than
edited (~200-line diff that reads as churn but is forced by
`grammar/tests.rs:117-126`). Verified from both spec files that the two structural
assertions at `ebnf.rs:126-139` still hold: seven `wf-ebnf` fences, 86
productions, even though per-fence counts move (GRAM-2 30→29, GRAM-4 30→27,
GRAM-5 16→17, EFF-1 3→6). Landing the spec bytes here also settles the version
number and turns `spec-append-only` green.

**P3 — Conformance next, out of dependency order, because it needs no build.**
`validate_manifest` (`runner.py:236-330`) rejects the first case citing a rule not
defined at a line start, so all 357 rows citing a retired tag are hard errors the
moment v0.60 is active; `coverage` (`:332-351`) then fails on any of the 19 new
rules with neither a case nor an annotation. Both are Python. Landing this while
the build is red converts an otherwise idle stretch into a green stage and forces
the retirement justifications to be written while the reasoning is fresh, rather
than under end-of-port pressure. The adapter's verdicts stay unproved until P12;
that is expected, not a gap.

**P4 — Resolution, because the semantic core matches on its enums.**
`roles.rs` matches on eight `Production::` variants and five `FixedTerminal::`
variants generated in P2; nothing in `roles.rs` or `scopes.rs` compiles until
those exist. The prelude rides here because it is data, not a table:
`prelude.rs::DECLARATIONS` is `.wf` source run through the real pipeline, so 20
new records plus the `&uniq`→`&`, `Slice`/`MutSlice`→`&[T]` and
`len_of(deref(x))`→`x.len` rewrites of the existing 29 signatures must land with
the resolver that reads them — and **D1 blocks this package**, not a later one.

**P5 — The rule table before any checker module.** Forced by the compile-time
rank assertions (see above). `semantic/mod.rs` has two hand-maintained chains,
`next_in_definition_order` and `definition_rank`, that must both be rebuilt to
v0.60 definition order; `definition_rank_matches_the_active_specification` reads
the spec text and must now find 125 base rules where it found 129. `model.rs` and
`places.rs` ride along because every later package types against them.

**P6 — References before everything that consumes reference validity.**
This package is where the port is most exposed to going *permissive*: deleting
`check/borrows.rs` removes several hundred refusals at once
(`check_persistent_loan_access:2266`, `check_child_reborrow:1604`,
`check_returned_reborrow_lifetime:1838`) and the REF-2 validity flow must be
wired to every consumer in the same change. `REF-2`'s closing sentence puts
validity outside the entailment fragment `[ENT-1]`, so it cannot reuse
`entailment/flow.rs`'s L0 machinery and needs its own per-edge lattice.

**P7 — Effects and calls after references, because EFF-5 clause 3 reads them.**
EFF-5 clause 3 (a live outside reference whose path has a proper prefix among the
call's write paths becomes invalid) is a query into P6's validity state. The new
`calls/effects.rs` is the single most load-bearing new judgment in the port and
has **no predecessor to diff against**: nothing in v0.59 computes the pairwise
comparison of a substituted row, so there is no differential oracle, and the
conformance corpus that would be one is being rewritten in the same amendment.

**P8 — Windows after effects, because OP-10's rows are effect rows.** The nine
window operations become ordinary PRE-1 records, so their signatures leave the
checker; what stays is which operation invalidates which reference and what `W`'s
admitted argument set is. `WIN-2`'s overlap answers are a fixed table of six
statements, five of which are data and one of which (`r[i]` vs `r.last`) is
conditional on proving `i != r.len - 1`.

**P9 — Permission last in semantic, because it consumes everything.** PAR-1
becomes adjacency plus pairwise composition, which deletes `interposed_of`
(`permission.rs:894-1029`) and the whole window apparatus. It goes last because a
permission verdict never changes acceptance, so a wrong answer here is invisible
to every other package's tests, and it needs EFF-5's substituted rows and OWN-7's
widened overlap relation to be settled first.

**P10 / P11 — Lowering then backend, closing the build.** `backend/emitter/runs.rs`
is the critical path: five of the nine `[OP-10]` operations have no code today and
three must handle a `Ring` whose logical window wraps into two physical extents.
`emit_full_array_conversion` (`backend/emitter/array.rs:62`) already writes that
two-extent copy and is the only worked example in the tree — reuse it rather than
re-deriving the arithmetic. `driver/pinned_sentences.rs` is updated **last** in
P11, from the new DIAG-1 rank and role tables, with the row count recorded before
and after: it is the only completeness check on DIAG-1 and roughly a third of its
rows lose their rule, so updating it by deleting failing rows makes the check
vacuous with nothing failing.

**P12 / P13 — Tests after the build, in that order.** P12's semantic and backend
suites are the first real evidence any of P5–P11 is right. P13 comes after because
`compiler/tests/programs/*.rs` assert program *outputs* and are the oracle for
whether a `.wf` rewrite preserved meaning — a rewritten program whose output moved
was rewritten wrongly. P13 also unblocks
`finalize/tests/corpus_shape.rs:202-226`, which reads every `.wf` under
`tests/conformance/cases` and `tests/programs` and panics if a root is empty: the
syntax subsystem cannot go green before the corpus lands, regardless of how correct
the syntax code is.

**P14 — Docs last, and it is the only subsystem with no mechanical backstop.**
Verified: the sole `make check` target that reads `docs/` is
`spec-prose-integrity` (`Makefile:173-186`), which greps for a 64-hex digest and an
"active vN.M" sentence. A `docs/patterns.md` still teaching `region { }`, `&uniq`,
`dispose`, `replace`, `slice_of` and `len_of` passes the gate exactly as it does
today. Seven patterns retire (P3, P10, P20, P24, P25, P26, P32) and three are new
(early release by moving into a consuming function; `free_empty` on a proved-empty
linear window; `program no_heap;`). Close with one grep sweep over `README.md`,
`docs/*.md` and `design/**` for the retired spellings and every retired tag — an
explicit one-shot, deleted after use.

---

## 3. Owner rulings needed

Deduplicated from 62 rulings across the seven maps. Five nodes were named by four
or more maps independently; those are marked **(×N)**.

### 3.1 Blocking — settle before P2

| # | Ruling | Recommendation |
|---|---|---|
| B1 | **D1.** `[PRE-1]`'s bare `<T>`/`<W, T>` records do not parse under `[GRAM-2]`. Add an unbounded `gparam` alternative, give the records explicit bounds, or make the compiler-owned window parameter a distinct non-`gparam` spelling? | Distinct non-`gparam` spelling for `W`/`X` (OP-10 already says no source declaration can write one) **plus** explicit bounds on `T` — the narrowest change, and it keeps `gparam`'s mandatory bound. |
| B2 | **D2.** `Array<T, N>` versus `Array<T, n>`. | Confirm `N` is a prose metavariable; the parser is already right and the PRE-1 fence already writes lowercase. No grammar change. |
| B3 | Is the declaring `set` target intentionally gone? v0.59 LIV-2 let an unresolvable bare `set` target declare a binding; v0.60 SET-1 says nothing and the phrase appears nowhere in the file. | Yes, intentionally gone — but SET-1 should say so, because `set x = …` now gets an unresolved-name rejection with no restructuring text. Deleting the fixed-point promotion loop (`engine.rs:208-224`) removes resolution's only re-entrant path. |
| B4 | Do the pinned generated-table counts stay pinned? `grammar/tests.rs:18-19` pins `DECISIONS.len()==136` and `SELECT_ROWS.len()==6_827`; both change and neither can be computed without running the generator. | Keep them exact. The structural checks in the same file (`:129-151`, `:206-253`) prove the properties, but an exact pin is what makes a future amendment *notice* the move. Record the before/after in the PR. |

### 3.2 Design-tree contradictions — a line enters or leaves only by ruling

| # | Node | Ruling |
|---|---|---|
| T1 **(×6)** | `design/compiler/view-loans.md` | Retire. Its decision names a loan identity with a protected place, region, strength and parent holder; all four are abolished. **Recommendation: replace, not just delete**, with one successor decision on how the checker carries a reference's path identity across copies, joins and calls — `REF-1`'s join rule leaves that open and no other node would own it. |
| T2 **(×4)** | `design/compiler.md` lines 11 and 13 | Absorbed into the language: v0.60 `[OWN-7]` states the captured endpoints are immutable mathematical values, and `[PAR-2]` (`spec:1946-1948`) states the fixed finite image family and both retained sign goals. **Recommendation: narrow to what the spec does not fix** (which `ProofContext` receives the obligations, and that retention is the ordinary derivation ledger) rather than delete or keep verbatim. |
| T3 | `design/compiler.md` line 15 | Delete. Callee-result views cannot exist — FN-1 returns owned values only and REF-3 forbids returning a reference — and the parameter-origin half is subsumed by REF-1 and PAR-2. |
| T4 | `design/compiler.md` line 17 | Keep, replacing "loan duration" with reference validity under REF-2. Substance unchanged. |
| T5 **(×3)** | `design/compiler/cleanup-traversal.md` | Re-ground on `[STOR-8]`'s written no-heap declaration, and rule on its conflict with the owner's own follow-up (`docs/todo.md:198-205`): the node rejects a worklist "because the worklist allocated", but a worklist built from the freed cells allocates nothing, so the rejection's discriminating reason does not reach it. |
| T6 **(×3)** | `design/compiler/storage-placement.md` | Keep with a re-grounded reason. `[STOR-7]` now guarantees no judgment depends on a stable address, which supports the decision; but `WIN-3` makes a move out of a field consume the whole owner, so confirm the field-reuse admission still holds. |
| T7 **(×2)** | `design/compiler/resource-exhaustion-floor.md` | Keep, re-grounded: `[STOR-8]` (`spec:761`) now states heap exhaustion terminates from the trusted base, so the node should state only what it still owns — the record's fields and the stack half. |
| T8 | A new `design/compiler` node for `[DIAG-2]` attributes | **Yes.** DIAG-2 newly permits handing checker disjointness facts to the backend as attributes and metadata; `backend/abi.rs:3-4` currently states the *opposite* as a design fact. A performance choice the spec leaves wholly open with no owning decision — it should have one before P11. |
| T9 | `docs/why-whitefoot.md` (748 lines) | Freeze behind its existing banner. It is already stale against v0.59 (line 268 lists a `traps` effect that EFF-4 does not admit), so the port neither breaks nor repairs it; updating it is a separate essay, not port work. |
| T10 | `lib/containers` | `[OP-10]` puts `place_back`, `insert_at`, `remove_at`, `append`, `split_off` and `grow` in the language over `Box<Slots<T>>` — exactly what `GrowVector` provides. **Recommendation: repoint at a container the kernel still lacks** (ordered map, hash map) rather than retire; retiring also removes the `libraries` correctness group and changes the CI matrix. |
| T11 | `tests/codegen` (95 files, 2,027 lines) | Not in the gate, cannot break, but will be the largest block of dead v0.59 syntax in the tree. **Recommendation: freeze with an explicit note naming it v0.59 evidence.** |

### 3.3 Performance-relevant implementation choices the spec leaves open

| # | Choice | Recommendation |
|---|---|---|
| I1 **(×4)** | **May `places.rs` call into the entailment fragment?** Its stated invariant (`places.rs:1-14`) is that the relation is purely syntactic so every consumer sees one answer; `[OWN-7]` now decides two index steps by the fixed ENT-6 families under MSR-4's disposition. Options: on-demand query with a `&ProofContext`; pre-recorded obligations extending `record_range_separation:347-351`; proof-carrying offsets established at formation. | On-demand query **plus** a per-function overlap memo keyed on `(path, path)`. OWN-7 now runs at five consumers where v0.59 ran it at one (`commit_index_alternatives:770`), so without the memo the cost is one ENT-6 traversal per consumer per pair. The memo needs an explicit extension of `design/compiler/proof-query-context.md`. |
| I2 **(×3)** | **How much of PAR-1's composition the checker materializes.** "Pairwise means every ordered pair in the run" is O(n²) path comparisons per block, each possibly submitting ENT-6 goals. | Adjacent pairs plus transitive extension, with the full table behind a flag. It also changes the permission ledger's output shape, which `driver.rs:977-1120` asserts on. |
| I3 **(×3)** | **Representation of `Box<Array<T>>`, `Box<Slots<T>>`, `Box<Ring<T>>` and `&[T]`.** `[WIN-1]` says `len` is stored with the block; `[OP-9]`'s ceiling table prices `Slots<T>` at (24,8). Those describe a fat descriptor beside the slots *or* a thin pointer to a header. Both are under the ceiling, so `[STOR-6]` admits both. | Decide in `design/compiler` **before P10**. It fixes how many allocations `box_slots_new` makes, whether `grow` can use `realloc`, and how many loads a subscript costs. Ten `_host.ll` drivers hardcode `{ptr, i64}` today and `compute-regression.yml:91-93` measures against them, so a change is link-visible and invalidates every stored digest. |
| I4 | **`Slots<T, N>` and the head word.** Every constant-capacity run is `{ [N x T], i64 len, i64 head }` today and every boundary row reads `head` unconditionally; `[OP-9]` gives constant-capacity `Slots` one word, not two. | Split `Slots` and `Ring` into two IR types with two emitters. Keeping a dead `head` word costs the writer memory they did not ask for and hides a ceiling disagreement. |
| I5 | **Release-walk depth.** `[STOR-3]`/`[PROV-6]` say "one walk" and nothing about depth; `[STOR-8]` makes allocation total, so a deep `Box` chain is trivially buildable. `cleanup-traversal.md` accepted native recursion. | Ruled by T5. Whichever way it goes, `backend/tests/heap_programs.rs` (131 lines, the only coverage) needs a depth case. |
| I6 | **Which `[DIAG-2]` attributes are emitted, and on what evidence.** | Start with `nsw` from discharged `IntegerDomain` obligations only — the one channel with a directly available proof and no aliasing subtlety. Defer `noalias`: EFF-5 proves disjointness *at a call* while `noalias` is a callee-side promise about *every* caller, which is not the same statement. |
| I7 | **`grow`'s `realloc`.** The module declares only `malloc` and `free`. | Malloc-memmove-free first; `[STOR-7]` makes the copying route legal, so this is a pure performance choice with no correctness lever. Revisit with a measurement. |
| I8 | **Representation of REF-2 validity**: a separate pass, a lattice threaded through `check/control.rs`'s statement walk, or a lazily consulted side table. | Threaded lattice. Scope exit already visits the right edges, and REF-2's invalidation set is exactly enumerated. |
| I9 | **Bound on a reference variable's join path set.** REF-1 unions the incoming path sets with nothing bounding the result; indices are captured values and the count of distinct captured values across loop iterations is not obviously bounded. | Close the fixed point over the *static* path shapes and fall back on OWN-8's reject-when-unsure for the index components. Needs a written amendment either way. |
| I10 | **`r.last` disequality cost.** `[WIN-2]` fixes every overlap answer as data except one: a live `r[i]` overlaps `r.last` unless `i != r.len - 1` is proved — a goal on every element access meeting an `r.last` effect path. | Submit only when an `r.last` entry is present on the other side of the comparison. |
| I11 | **Where the FN-4 refinement query runs.** v0.60 changes FN-4 from structural equality to refinement by a fixed finite check, with no program point and no entering context, so none of `proof-query-context.md`'s preconditions hold. Quadratic in (formal × actual clauses) at every function-kind binding site. | Reuse `check/publication.rs`'s existing declaration-level closure as the shared engine, and extend the decision to a declaration-context query kind. |
| I12 | **Where EFF-5 is checked** — checker or entailment flow. Substitution exists in the checker (`calls/user.rs:1303`), kills in the flow (`flow.rs:6252`), and they already duplicate the filter-and-extend logic. | Checker, with the flow querying it. Clauses 2 and 3 need argument spellings and live reference state, which only the checker has. |
| I13 | **`[EFF-3]`'s allocation carve-out has no IR carrier.** `pure` is `EffectSet::NONE` and carries no allocation bit; `allocates` — the only thing that recorded allocation — is being deleted, yet EFF-3 still excepts a call that allocates from deduplication and reordering. | Keep a per-call `allocates` flag in the checked function metadata. Rederiving it at lowering from the callee set makes lowering a second acceptance path, which DIAG-2 forbids. |
| I14 | **`MeasureCell::Bounded`** carries `#[allow(dead_code)]` and "no row of this version's table selects bounded". v0.60's MSR-1 selects it for a `Ring`'s head and states no operation re-establishes it exactly. | Keep it as one fact shape, not two ordinary bounds; it is the first two-sided-only measure and its closure size is measurable. |
| I15 | **Whether the backend also asserts the no-heap mode.** `has_heap_storage` (`emitter.rs:254`) infers the answer today and can never disagree with itself, so a checker bug passes silently. | Assert. It is cheap independent evidence; accept the new backend failure mode. |
| I16 | **Does the backend still receive only call-rooted groups?** `IrBuilder::overlaps` keys every group on a call `NodePath`, and `two-worlds.md` derives the clone set from the call graph *and* the permission table; v0.60 PAR-1 permits adjacent statements, most of which are not calls. | Keep exporting only call-rooted runs. The backend has no hand-out lowering for anything else and the two-worlds decision is owner-approved with measured numbers behind it. |
| I17 | **Element maps through a `Box` deref.** `[TYPE-9]` puts every runtime-capacity shape inside a `Box`, so the realistic parallel kernel writes `deref(b)[a*i + c]`; whether the affine refinement survives the deref step is undecided by the spec. | Must survive, or the measured kernels stop being permitted. Highest performance-load-bearing open choice in the parallel subsystem. |
| I18 | **Do the 12 operation families and 8 measure names move into `build.rs`?** `catalog.rs` is a committed 98-entry array policed by an extraction test — the arrangement `design/compiler/build-inputs.md` rejected for parser tables, though not in its letter. The port touches all three tables. | Move them. The port is the natural moment and the reason behind the existing decision applies verbatim. |

### 3.4 Coverage and evidence rulings

| # | Ruling | Recommendation |
|---|---|---|
| C1 | **How the 19 new rules reach 100% conformance coverage.** `coverage()` fails on any uncovered rule and has exactly two instruments: a case, or a `{rule, covered_by, reason}` annotation. Several new rules are unexhibitable by construction — `[TYPE-10]` says the four window parts are names no program reads, writes or forms a reference to; `[STOR-7]` is a consequence statement with no source form. | Annotate `TYPE-7`→`TYPE-8`, `STOR-7`, `TYPE-10`; write cases for the other sixteen. Getting this wrong is invisible: the gate goes green either way. |
| C2 | **The 115 retired cases** (99 citing only retired tags, plus retirements among the 103 whose expected rejection rule retires). Each needs an honest technical explanation in the same change. | Five of them are **verdict flips**, not retirements: `inv1-neg-equality-relation` (INV-1 now admits `==`), `type7-neg-return-reference-holder` (TYPE-7: a reference is read bare), `form3-neg-region-param-missing-apostrophe`, `own5-neg-captured-index-loan` and `own5-neg-captured-range-endpoint-reassignment` (REF-1 evaluates the index at formation). **These must come back as positives**, not disappear — they are coverage holes `runner.py` cannot see, because coverage counts tags and not subjects. |
| C3 | **What replaces the allocation-failure population.** `[STOR-8]` makes allocation total; `backend/tests/exhaustion.rs` (1,123 lines) and `lib/containers/tests/vector_allocation_observer.c` are built on observable refusal. | Keep the trusted-base termination case and retire the source-observable refusal half. This moves gate wall time materially and should be measured. |
| C4 | **Where the new families' tests live.** `design/compiler/verification.md` already rules conformance vs programs vs compiler tests, and that a new WF-compiling test must identify the missing coverage. | Apply the existing decision deliberately per family rather than defaulting REF-*/WIN-*/OP-10..15 to compiler-test level. No tree change needed. |
| C5 | **PRE-1 diagnostic ordinals.** `resolution/tests.rs:2468` pins 279 records and six absolute ordinals; the estimate after the port is ~361. | Derive the expected ordinals from the spec's preorder sentence and fix the prelude source until they match — never the reverse. Say so in the PR; nothing mechanically blocks the shortcut. |
| C6 | **`[MSR-3]` has no SWAP or DISPLACED row.** Verified `spec:2473-2486`: REBIND, ELEMENT, CONSTRUCT, DESTRUCTURING, PAYLOAD only. So nothing carries a displaced value's measure out of a `swap`, and `docs/patterns.md:1391-1393`'s current advice has no v0.60 equivalent. | Confirm it is intended (the writer reads the measure and branches, as MSR-3 already prescribes after a window operation). P28 cannot be silent either way. |

---

## 4. Risks that could change the plan

**R1 — The port can go permissive without any test failing.** This is the
dominant risk and it appears in three independent places. (a) Deleting
`check/borrows.rs` removes several hundred refusals at once; if REF-2's validity
flow is not wired to every consumer in the same change, the intermediate compiler
accepts programs neither version admits, and the ownership tests pass because
their fixtures were rewritten in the same commit. (b) Permission failures never
change acceptance, so a statement form that falls through the merged per-statement
footprint routine contributes an empty footprint and **widens** permission — under
`--par` that is a data race no test detects. Two independent widenings land in the
same change (deleting the arena denial; kernel rows becoming ordinary). (c)
`WIN-2`'s six fixed overlap answers are data, not a proof; a wrong cell is
fail-open with no natural test. *Mitigation:* cut P6–P9 into commits that are
never permissive, and treat "an exhaustive match stayed exhaustive" as a reviewed
property rather than a compiler guarantee.

**R2 — The emptied-case class.** `runner.py`'s own `verdict_diff` docstring names
it: a positive case whose subject was deleted "still declares, and still reaches,
the verdict it always did; there is nothing here to compare", and reading a green
run as "the migration broke nothing" is wrong in exactly that direction. With 602
rewritten rows and 238 accept cases, plus 55,979 lines of Rust test fixtures being
retyped, this is the most likely way the port loses coverage silently. Nothing
reports "this module was ported"; the only signal is a green `make check`, which is
not the same thing. *Mitigation:* a per-module ported/not-ported ledger with a
stated deletion condition.

**R3 — `entailment/flow.rs` is 16,436 lines and the kill machinery is spread over
seven spans.** The `element: bool` carve-out alone is read at 3615, 4146,
4256-4290, 6168, 6296, 13311 and 15315, and `[MSR-2]` says the carve-out is
*removed*, not narrowed. Removing a flag seven independent sites read, in a file
that cannot be compiled until the whole port lands, is the highest-probability
source of a silently wrong kill.

**R4 — Four pinned numbers and ~20 pinned ordinals go wrong simultaneously.**
`86 / 136 / 6827 / 111` in `grammar/tests.rs:17-20`, the production-index
assertions at `:21-106`, and the `FixedTerminal`-as-`u8` pins at
`terminal.rs:833-875`. Only two are derivable by reading (86 stays; the diagnostic
order becomes 101). The rest need a build, so the whole syntax test module is
uniformly red until P11 — a poor signal for whether the rest of the port is
correct, and easy to misread as a fixture problem when a strong-LL(2) conflict is
what actually happened.

**R5 — Three new judgments have no predecessor and no oracle.** EFF-5's pairwise
comparison, REF-2's validity flow, and PAR-1's state rule ("both statements'
paths are interpreted in the state before the first; the first's `ensures` maps
the second's indices into that state" — verified that neither `permission.rs` nor
`loop_permission.rs` contains the string `ensures` today). There is no v0.59
behaviour to preserve and the conformance corpus that would be the differential
oracle is being rewritten in the same amendment.

**R6 — FN-4's refinement check fails open, not closed.** The structural-equality
check it replaces (`check/behavior/contracts.rs:70`, one `!=`) fails closed; the
natural implementation bug in a refinement check — treating an undischarged query
as "no constraint" — accepts instead. It must be written to fail closed
deliberately.

**R7 — `_host.ll` ABI drift is invisible until the performance workflow runs.**
Ten hand-written LLVM drivers hardcode the current `buffer<T>` representation.
They are compiled by `compute-regression.yml` and `compute-bench.yml`, neither of
which is in `CHECK_GROUPS`. A representation change lands green locally and fails
— or silently miscompares — on the next performance run. Rewriting every compute
program also changes every stored `sha256sum` baseline, so that baseline must be
deliberately rebuilt, not accidentally regenerated.

**R8 — No measurement exists for the new overlap cost.** I1, I2, I10 and I11 are
all performance questions, and `research/experiments/` holds no baseline for the
v0.60 overlap judgment because the judgment does not exist yet. The port must
choose provisionally and record each choice as an amendment rather than as a
measured decision — and `docs/todo.md`'s "Performance floor after the port" is the
acceptance criterion this closes out.

**R9 — `program` and `no_heap` leave IDENT.** Verified the 43 current occurrences
of `program` under `tests/programs/*.wf` and `tests/conformance/cases/*.wf` are all
inside doc STRING bodies, so today's corpus is clean — but the check must be redone
after P3 and P13, and `no_heap` was never grepped for because nothing had it. The
failure is a grammar-shape error, not a reserved-word message.

**R10 — Docs and the design tree have zero mechanical backstop, and the compiler
tree currently carries no v0.60 ruling at all.** Verified `design/log.md:8-14`: the
2026-09-19 entry names 32 `language/*` nodes and **zero** `compiler/*` nodes. Until
T1–T8 are ruled, every compiler-tree line this port touches stays an amendment
beside the tree. Request them early; they cannot be batched to completion review.

**R11 — Five files straddle subsystem boundaries** (`places.rs`, `model.rs`,
`check.rs`, `flat_storage.rs`, `permission.rs`) and whoever ports the second one
finds the first port already changed its imports. The `flat_storage.rs` split in
particular (set-target and element-write half is ownership, shape half is types)
should be agreed before P6 starts, not discovered during it.

**R12 — `Ast::Terminal(Vec<String>)` becomes a one-element vector everywhere.**
Deleting `split_spelling`'s `&uniq` case leaves a vector type with no second
element. Collapsing it touches `model.rs`'s `Kind`, `grammar.rs`'s
`GrammarNodeKind::TerminalSequence`, `GrammarNode::terminals()` and the generated
`GRAMMAR_TERMINALS` arena — a real refactor with no spec pressure behind it.
*Recommendation:* leave it and note why, rather than absorbing an unrelated
refactor into the port.
