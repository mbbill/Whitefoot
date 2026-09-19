# Specification amendment map: candidate x1 into the next kernel specification

Basis. Candidate x1 is `research/investigations/access-effects/CANDIDATE-X1.md` (revision 5,
Rules 1-16, plus the "Not in this candidate" list, which this map treats as deciding deletions
exactly as a numbered rule does). The active specification read is
`/private/tmp/whitefoot-spec-x1/spec/kernel-spec.md`, 3599 lines. Nothing in that worktree was
modified. Fourteen section maps sit beside this file in the same directory, one per specification
range, and 32 drafted design amendments sit in `amendments/`.

How to read the counts. A "rule entry" is one row of a section map: a long rule such as `[FORM-2]`
or `[DIAG-1]` is counted once as a whole and again per clause where its clauses have different
fates, because the amendment edits clauses, not rules. A "case row" is one conformance case as
counted by one section map; a case tagged by several sections is counted once per section, so the
case totals are work items, not distinct files. The corpus has 1133 `.wf` case files and 1155
manifest lines; the de-duplicated corpus figure is at the end of section 1.

---

## 1. Totals

### 1.1 Specification rules and clauses

| Verdict | Count | Meaning |
|---|---|---|
| keep | 291 | survives as written; x1 is silent or agrees |
| rewrite | 192 | keeps its identity and job; its text moves to x1 vocabulary |
| replace | 56 | deleted, its job taken over by a different rule |
| delete | 76 | dies with no successor obligation |
| new needed | 63 | x1 states something no current rule carries |
| **total entries** | **678** | |

### 1.2 Conformance cases

| Fate | Count | Meaning |
|---|---|---|
| unchanged | 691 | source bytes and required verdict both survive |
| rewrite | 985 | source must be re-spelled; verdict normally survives |
| retire | 515 | the construct the case tests no longer exists |
| verdict changes | 34 | source survives essentially as written, required verdict flips |
| **total rows** | **2225** | over 1133 case files, so a case tagged in n sections appears n times |

### 1.3 Per section

| Specification range | keep | rewrite | replace | delete | new | cases unchanged | rewrite | retire | verdict |
|---|---|---|---|---|---|---|---|---|---|
| 2 Canonical form (25-174) | 14 | 6 | 0 | 8 | 3 | 29 | 3 | 46 | 0 |
| 3 Grammar (175-418) | 56 | 19 | 3 | 10 | 4 | 63 | 30 | 6 | 0 |
| 4 Types (419-669) | 6 | 7 | 1 | 0 | 3 | 91 | 130 | 43 | 2 |
| 5 Ownership, first half (670-950) | 1 | 8 | 6 | 5 | 6 | 33 | 72 | 233 | 8 |
| 5 Ownership, second half (951-1235) | 16 | 23 | 12 | 22 | 2 | 0 | 85 | 45 | 15 |
| 6 Storage (1236-1335) | 28 | 12 | 18 | 15 | 10 | 3 | 20 | 21 | 4 |
| 7 Operations (1336-1573) | 21 | 15 | 8 | 5 | 10 | 135 | 162 | 29 | 1 |
| 8 Functions (1574-1880) | 0 | 11 | 0 | 0 | 2 | 131 | 121 | 51 | 2 |
| 9 Effects (1881-1941) | 1 | 6 | 0 | 2 | 4 | 7 | 51 | 17 | 1 |
| 10-12 Errors, programs, diagnostics (1942-2404) | 60 | 24 | 2 | 1 | 5 | 28 | 12 | 1 | 0 |
| 13 Execution overlap (2405-2476) | 31 | 15 | 2 | 8 | 2 | 0 | 17 | 1 | 0 |
| 14 Prelude (2477-2697) | 43 | 26 | 0 | 0 | 7 | 31 | 75 | 3 | 0 |
| 15 Obligation discharge (2699-3540) | 9 | 19 | 4 | 0 | 5 | 135 | 206 | 19 | 1 |
| 16-17 Worked example, spec meta (3541-3599) | 5 | 1 | 0 | 0 | 0 | 5 | 1 | 0 | 0 |

### 1.4 De-duplicated corpus scale

Of the 1133 case files, 700 contain at least one construct x1 deletes and 433 contain none
(`&uniq` 440, `region` 489, a REGIONID 382, `Vector` 224, `FixedVector` 157, `Slice` 110,
`arena` 87, `Heap` 86, `allocates` 79, `MutSlice` 43, `buffer` 41, `dispose` 18, `box<` 5).
About 62 per cent of the corpus is touched. The per-section maps decide what each case is for;
they do not decide whether it still compiles, because almost none of them does.

---

## 2. Dependency-ordered sequence of edits

Twelve steps. Each is a coherent intermediate state: at the end of every step the specification
names no construct it has not yet defined, and cites no rule it has already deleted.

| # | Step | Sections and rules | Why this order |
|---|---|---|---|
| 1 | Type inventory and storage shapes | 4: `[TYPE-2]`, new `[TYPE-8]`/`[TYPE-9]`/`[TYPE-10]`; 3: the `[GRAM-3]` type fence | Every later rule names `Array`, `Slots`, `Ring`, `Box`, the measures and the reference kinds, so the type names and their two placements (constant capacity inline, runtime capacity only as `Box` content) must exist before any path, row, contract or operation can be spelled. |
| 2 | Region and store-brand excision | 2: `[FORM-8]`, `[FORM-3]`; 3: `[GRAM-1]`, `[GRAM-2]` region productions, `[GRAM-4]` `region_stmt`/`dispose_stmt`; 5: `[OWN-3]`, `[OWN-4]`, `[OWN-10]`, `[OWN-11]` region half, `[PROV-1]`; 6: `[STOR-4]`; 9: the `allocates` alternative and `[S23]`; 7: the `[OP-1]` effects column | This is the one step that is pure deletion with no successor, and it removes the vocabulary every later rewrite would otherwise have to carry through, so doing it before the rewrites halves the text each of them touches. |
| 3 | Path grammar and place forms | 3: `[GRAM-5]` `place`, `psuffix` (new range step and payload step), `pbase`, `borrow_expr`, and `&[T]` in parameter position; 2: `[FORM-2]` rendering of the new spellings | A path is the term every later rule is stated over, so `&r[i]`, `&r[lo..hi]`, `deref(b)` and a payload step must derive before any rule may cite one. |
| 4 | References: identity, validity, no escape | 5: new reference rules for Rule 2, Rule 3 and Rule 4; replace `[OWN-2]`, `[OWN-5]`, `[OWN-6]`; delete `[OWN-14]`, `[VIEW-1]`, `[VIEW-2]`, `[VIEW-4]`, `[VIEW-6]`; rewrite `[OWN-7]` (overlap) and `[OWN-13]`; 4: `[TYPE-7]` | The path grammar now exists, and validity must be a stated fact before any rule may say that a write, a move or a call invalidates a reference, which steps 5 to 9 all do. |
| 5 | Storage operations and how a value leaves storage | 6: `[STOR-1]`, `[STOR-2]`, `[STOR-3]`; 5: `[BLK-0]`, `[BLK-1]`, `[BLK-2]`, `[BLK-3]`, `[BLK-4]`, `[PROV-6]`, `[LIV-2]`; 4: `[SET-1]`, `[SET-2]`; 7: the `[OP-1]` table plus the ten new operation rules | The window operations are stated as reference parameters with effect rows over measures and window parts, so they need the shapes of step 1 and the paths of steps 3 and 4; they are written before the effect grammar so that the row grammar has real rows to be checked against. |
| 6 | Effect rows | 9: `[EFF-1]` (paths root at reference parameters; the meaning of `writes`), `[EFF-2]`, `[EFF-3]`; the allocator axiom (one heap, allocation carries no entry) | Rows are written over paths and over the measures and window parts, so the row grammar follows steps 1, 3 and 5; it precedes the call rule, which is stated entirely over substituted rows. |
| 7 | The call-site rule | 5 or 9: the new rule for x1 Rule 10; replace `[OWN-12]`; rewrite `[CALL-1]`, `[CALL-3]`, `[CALL-5]` | It consumes three things at once, the row grammar of step 6, the overlap judgment of step 4 and the invalidation events of step 4, so stating it earlier would forward-reference all three. |
| 8 | Contracts, facts and the proof system | 8: `[FN-8]`, `[FN-9]`, `[MSR-5]`, `[CALL-4]`, `[CALL-6]`, plus the refinement check and the widened result route; 15: `[ENT-2]`, `[ENT-3]`, `[ENT-5]`, `[ENT-6]`, `[MSR-1]`-`[MSR-4]`, `[INV-1]`, `[PRF-1]` | Facts are stated over paths and measures and are killed by declared writes, so the fact system can only be closed after the rows of step 6 and the call rule of step 7 fix what a write is and where it reaches. |
| 9 | Overlapped execution | 13: `[CAP-1]`, `[PAR-1]`, `[PAR-2]` | Rule 13 is defined by explicit reference to the call-site judgment ("the same path-overlap and index/range-disjointness judgment as Rule 10"), so it can only be restated once step 7 is fixed. |
| 10 | Prelude and host surface | 14: `[PRE-1]` signatures over `&` and `&[T]`, the `Oom` nominal, the allocation and window-operation records, the entry signature without a store parameter | Prelude entries are ordinary declarations checked by the same rules, so they must be written in the finished vocabulary of steps 5 to 8, not before it. |
| 11 | Errors, programs, diagnostics | 10-12: `[ERR-3]`, `[ERR-4]` classification of the new rejection classes, `[PROG-3]` ambient heap, `[DIAG-1]` enumerations and ranks, `[DIAG-2]` retention | `[DIAG-1]` enumerates every declaration class and pins every rejection to one numbered rule, and `[DIAG-2]` retains every checked operation, so both can only close after the last rule is minted and the last one deleted. |
| 12 | Worked example, meta checks, conformance corpus | 16: `[EX-1]` bytes; 17: `[META-1]`-`[META-5]`; `tests/conformance` cases, manifest, coverage | `[EX-1]` is byte-exact canonical form, so it is written last; and `runner.py` fails on a citation to a deleted tag and on an active rule with no coverage, so the corpus work closes the change. |

---

## 3. Consolidated deletions and additions

### 3.1 Every deleted mechanism and the rule tags it takes with it

| Deleted mechanism | Deciding x1 rule | Rule tags it takes with it |
|---|---|---|
| Lifetimes, regions, `region` statements, region parameters | Rule 14; "Not in this candidate" | `[FORM-8]` entire, the REGIONID class in `[FORM-3]`, the region token in `[GRAM-1]`, `region_params` and `region_param` in `[GRAM-2]`, `region_stmt` in `[GRAM-4]`, the REGIONID `targ` in `[GRAM-3]`, `[OWN-3]`, `[OWN-4]`, `[OWN-10]`, the region half of `[OWN-11]`, the REGIONID row of `[TYPE-6]`, the region-edge clause of `[STOR-3]`, `[STOR-4]`, the REGIONID rows and ranks of `[DIAG-1]` |
| Store brands, providers, capabilities, `dispose` | Rule 5, Rule 8, Rule 14 | `[PROV-1]` with its store/provider/run table, the capability half of `[PROV-6]`, the store rows and reservation clauses of `[BLK-2]`, `dispose_stmt` in `[GRAM-4]`, the `arena_new` row of `[OP-1]`, the per-argument written-argument criterion in `[BLK-0]`, the nine citations of the undefined tag `[S39]` |
| Permission markers, loans, holders, child reborrows, suspension | Rule 2, Rule 3, Rule 9 | `[OWN-2]`, `[OWN-5]`, `[OWN-6]`, `[OWN-9]`, `[OWN-12]`, `[OWN-14]`, the binder-mode half of `[OWN-13]`, `mode` alternatives in `[GRAM-3]`, the `&uniq` expansion sentence of `[GRAM-1]`, the ruling of record and the deferred two-axis delta in `[LEX-1]`, the `&uniq` term in `[CAP-1]`'s vocabulary list, the loan-denial matrix and loan clauses of `[PAR-1]` and `[PAR-2]` |
| Slice and view values | Rule 7; "Not in this candidate" | `[VIEW-1]`, `[VIEW-2]`, `[VIEW-4]`, `[VIEW-6]`, the `slice_of` and `mut_slice_of` rows of `[OP-1]`, `Slice`/`MutSlice` in `[TYPE-2]` and `[GRAM-3]`, `[CALL-3]`, the slice-result origin ceiling of `[FN-1]`, the region-bearing relation of `[STOR-5]` |
| The `allocates` effect category | Rule 14 | the `allocates` alternative of `[EFF-1]`, `[S23]`, the allocation clause of `[EFF-3]`, the effects column of the `[OP-1]` table, the allocation term of `[PAR-1]`'s footprint, every `allocates` entry in `[PRE-1]` |
| The static allocation-fit predicate and its obligation | Rule 14 (subject to open question 12) | the `buffer_fits` row of `[OP-1]`, the predicate and obligation halves of `[OP-9]`, the AllocationFit family of `[ENT-6]`, the allocation-fit class in `[ERR-4]` and its retention in `[DIAG-2]` |
| Retired storage names `buffer`, `FixedVector`, `Vector<'s,T>`, `arena<'r,T>`, `box<T>`, `array<T,N>` | Rule 5, Rule 6, Rule 16 | those alternatives in `[TYPE-2]` and `[GRAM-3]`, `[BLK-1]`'s two-run inventory, the `[BLK-2]` formation rows, the `box_new`/`buffer_new`/`buffer_vacant`/`array_new` rows of `[OP-1]`, the class table of `[STOR-1]`, the per-type leaves of `[STOR-3]`, the measured-type table of `[MSR-1]`, the indexable domain of `[OP-4]` |
| The `len_of` measure-reader family | Rule 6 | the four measure rows of `[OP-1]`, the four formers of `[MSR-1]`, the former clause of `[MSR-5]`, the "not in this domain" clause of `[BLK-0]`, the `affine_factor := call` motive in `[GRAM-4]` |
| Takes, holes, partial moves, read-out, the multi-target commit | Rule 6, Rule 12 | the read-out machinery of `[LIV-2]`, the statement form behind `[SET-2]` and `replace_let_rhs` in `[GRAM-4]`, the affine-target hard error of `[STOR-1]`, the partial-consume refusal of `[PROV-6]` |
| Refusals that x1 reverses | Rule 6, Rule 16 | `[BLK-3]`'s sentence refusing swap, exchange, growth, clear, truncation and middle removal; `[STOR-1]`'s rejection of the arena-index-pool pattern and the keyed-collection block that follows it; `[OWN-7]`'s literal-index restriction on separating two indices |
| The window mechanism of overlapped execution | Rule 13 | `[PAR-1]`'s member-form enumeration, its intervening-statement window, its exit-edge denial, its non-call-borrow clause and its normal-continuation clause |

### 3.2 Every new rule x1 requires, and the section it belongs in

| Section | New rules |
|---|---|
| 2 Canonical form | a clause stating that the existing attachment sets render x1's spellings (`&p`, `&deref(b).f`, `&r[i]`, `&r[lo..hi]`, `&[T]`, `Box<Slots<T>>`, `move deref(b)`, the measure and part members); a line-bearing entry for the destructuring consume's rest form; a `[FORM-3]` ruling on whether the eight measure and part names are reserved from field binding |
| 3 Grammar | the range step `[lo..hi]` in `psuffix`; an enum-payload path step; the range-reference type `&[T]` in a parameter-only position; the rest form of the destructuring consume; `rtype` narrowed to owned |
| 4 Types | `[TYPE-8]` reference kinds are not value types and no aggregate or type argument may hold one; `[TYPE-9]` the three shapes, their two placements, and `Box<T>` as one heap object; `[TYPE-10]` measures as read-only pseudo-fields and window parts as names occupying no declaration domain |
| 5 Ownership | a reference is a local name for a path (Rule 2); reference validity is a fact with its invalidation events (Rule 3); references never escape (Rules 1, 4, 15); the call-site rule (Rule 10); the built-in exchange and the atomic in-place update (Rule 6); pools and arenas are usage, not types (Rule 16) |
| 5 and 6 Storage | the four window parts as row and overlap vocabulary; runtime-capacity shapes only as `Box` content; `insert_at`, `remove_at`, `append`, `split_off`, `grow`, `set`, `replace`, `place_front`, `take_front`; `free_empty`; the construction set; how a value leaves storage; relocation by byte copy; the allocator axiom |
| 7 Operations | ten rules: the window operations, `swap`, the atomic update, the constructors, move-out, range-reference formation, the measure read, the window parts, `free_empty`, and release on assignment |
| 8 Functions | a deterministic, terminating check for "row is a subset, `requires` weaker, `ensures` stronger" at a function-typed parameter; a widened result route admitting `ensures when Err:` and a unit payload |
| 9 Effects | the pairwise call-site check; the meaning of `writes` (writing, replacing, moving out of, freeing, closed downward); the allocator axiom; the row vocabulary of measures and window parts |
| 10-12 | `[ERR-4]` classification of the new rejection classes (invalid reference, unproved overlap); the `[PROG-3]` ambient-heap sentence; `[DIAG-2]` retention of recorded reference paths, validity facts, the atomic update, and the overlap derivation; a `[DIAG-2]` ruling on handing disjointness facts to the backend |
| 13 Overlap | the state in which two statements' paths are interpreted; allocation and release contribute no path |
| 14 Prelude | the `Oom` error nominal and its paired payload shape; the allocation and construction records; the window-operation records; `swap`; `free_empty`; the measure pseudo-fields; the window parts; `&[T]` |
| 15 Obligations | window-part terms and their overlap answers; variant-refinement facts from a match arm; reference-validity facts; the fact image of the atomic in-place update; `free_empty`'s discharge |

---

## 4. Open questions for the owner

Each is a yes/no or a choice. The recommendation is the reader's where one was given; where the
readers disagreed or gave none, the row says so.

### Surface and spelling

| # | Question | Recommendation |
|---|---|---|
| 1 | Does `pure` survive as the spelling of the empty effect row? x1 never writes the word, and Rule 14 makes an allocating function's row empty. | Yes, keep it; x1 is silent. If no, roughly 135 cases marked unchanged in section 7 and every prelude signature change bytes. |
| 2 | Does `deref(p)` survive on a plain `&` reference, or is `deref` only the `Box`-content step with a reference read directly? | Box-content only. Every x1 example reads a reference parameter bare; the change touches most of the corpus and eight verdict flips in section 4. |
| 3 | Does the statement form `let x = replace p = e;` survive beside the built-in `replace(&r[k], x)`? | No, retire the statement: two spellings of one construct is a `[FORM-1]` defect. |
| 4 | Does the multi-target `set (a, b) = ...` survive beside the built-in `swap`? | Retire it; but `liv2-pos-swap-and-rotation` exercises a three-target rotation that `swap` alone cannot express, so the owner should confirm what replaces it. |
| 5 | How is an enum payload step spelled in a path and in an effect row? x1's `n.left.Some.0` matches neither `[GRAM-5]`'s `psuffix` nor `[GRAM-10]`'s named-binder discipline, and `[EFF-1]` today excludes payloads from effect paths. | None; x1 fixes only the meaning. A spelling must be chosen in step 3. |
| 6 | Where does `&[T]` live: a `[GRAM-3]` `type` alternative, or a parameter-only position? | Parameter-only, so Rule 1 and Rule 15 stay syntactic. |
| 7 | Does `rtype` keep `own` written, or is the mode dropped from result bindings? | Keep `own` written; dropping it rewrites every signature in the corpus for no semantic gain. |
| 8 | Are the eight names `len`, `cap`, `room`, `head`, `next`, `last`, `filled`, `free` reserved from field binding, as the five mode suffixes already are? | Yes; otherwise `r.len` carries two meanings, which `[FORM-1]` forbids. |
| 9 | What spelling do the `Type::name` constructors take (`Box::new`, `Slots::new<T, N>`, `Slots::from_array`), given `[FN-5]`'s ban on methods and receivers and `[BLK-0]`'s named-argument discipline? | None; this is a real grammar decision, because x1 declares its own notation to be pseudocode. |
| 10 | Is `Array` a nominal in `[TYPE-6]`'s domain, replacing the lowercase built-in container convention (`array`, `buffer`, `box`)? | Yes, follow x1's capitalization, and retire the lowercase container convention with it. |
| 11 | Is the array element-list literal `[a, b, c]` admitted in expression position, not only in a `cvalue`? | Yes; the rendering is already settled by `[FORM-2]`, the admission is a `[GRAM-5]`/`[CONST-2]` decision. |

### Storage, allocation and operations

| # | Question | Recommendation |
|---|---|---|
| 12 | Does the static allocation-fit obligation survive Rule 14's runtime `Result`? | None; the readers split. It decides `[OP-9]`, `[ENT-6]`'s AllocationFit family, `[ERR-4]`'s classification, `[DIAG-2]`'s retention and six cases. |
| 13 | Does `grow` re-run `[STOR-6]`'s target-layout ceiling check, being the only operation that resizes an existing allocation? | Yes, or state explicitly that it does not; x1 is silent and the current text covers initial materialization only. |
| 14 | Does `replace` apply to any owned place, or only to a window slot? x1 lists it under window operations, then names only `swap` and the atomic update as general. | Keep it general, as `[SET-2]` is today; `set2-pos-affine-field-replace` exercises exactly that. |
| 15 | Is `replace` over a copy place still rejected? | Yes, keep the refusal; x1 is silent. |
| 16 | Is a range reference into a wrapped `Ring` admitted, refused, or admitted under a proved non-wrap premise? A wrapped window is two extents, and `&[T]` has one `len`. | Refuse it or carry the existing non-wrap premise; do not leave it implicit. |
| 17 | Is `truncate` a built-in row or library code? x1 lists it as library yet names its `ensures` beside `take_back`'s. | Library, and state that a library `ensures` carries no slot bound across a call. |
| 18 | Does `grow`'s `Result` refusal reopen `[BLK-3]`'s refusal of operations that are total at a capacity boundary (a growing `push` in the kernel)? | No; keep the refusal and keep growth in the library. |
| 19 | Where does the runtime-length `Array`'s length word live, and does `[STOR-1]`'s prohibition on stored length narrow to `Array` only? | Narrow the prohibition to `Array`; `Slots` and `Ring` store `len` (and `head`) by design. |

### References, effects and calls

| # | Question | Recommendation |
|---|---|---|
| 20 | Is an effect row exact in both directions, or only a covering bound? Rule 9 says "covered by", Rule 10's parenthetical says a row states only what the body does. | Keep exactness, as `[EFF-2]` has it; the answer also fixes what "subset" means at a function-typed parameter. |
| 21 | Does a row that names both `reads(p)` and `writes(p)` for one parameter self-conflict under Rule 10's pairwise check? Ten prelude records are written that way. | State that `writes(p)` subsumes `reads(p)`, so the pair is never written; otherwise most prelude signatures must change shape. |
| 22 | Does a canonical category order survive, and may a category appear more than once in one row? x1's own rows write `writes(r.next), writes(r.len)`. | Drop the once-per-category clause; keep or drop the order, but say which. |
| 23 | Must an operation that observes prior state while changing it name both categories? `[EFF-1]` says yes; x1's `take_back` and `replace` return the old value and declare only writes. | Drop the clause; x1's own rows violate it. |
| 24 | May a function body move out through a reference parameter? Rule 9's `writes` covers "moving out of"; `[OWN-5]` forbids it outright. | Admit it where the row declares the write; two ownership cases sit exactly here. |
| 25 | Where does the call-site rule live, section 5 or section 9? | Section 9, with section 5 citing it, because it is stated entirely over substituted rows. |
| 26 | Which section owns the window parts, the measures, and the allocator axiom? | Storage owns the parts and measures (they follow from the shape); effects owns the allocator axiom's "no entry" half. |
| 27 | Does the empty row still license deduplicating or reordering a call when the callee allocates? Allocation failure is observable even though addresses are not. | No; exclude an allocating call from the dedup licence, or say why `Oom` is not observable enough to matter. |

### Proofs, facts and overlap

| # | Question | Recommendation |
|---|---|---|
| 28 | How is "requires weaker, ensures stronger" decided without a solver? `[FN-4]` today demands structural equality and the project forbids SMT for acceptance. | A fixed finite check over the `[ENT-1]` affine fragment; it must be written, or Rule 10 is unimplementable. |
| 29 | Does `[INV-1]`'s refusal of `==` and `!=` in an invariant target stand? x1's own worked example writes `invariant h: r.len == n - i;`. | This is a direct conflict between x1's example and a rule x1 never mentions; the owner must choose. |
| 30 | Is a reference-validity fact inside `[ENT-1]`'s fragment, or an ownership judgment outside it? | Outside the affine fragment, as an ownership judgment; otherwise `[ENT-1]`'s state gains a fourth component. |
| 31 | Do measures keep an `[OP-1]` reader row at run time, or become purely a place form? | Purely a place form; one quantity, one spelling. |
| 32 | Does a `match` arm's variant refinement fact get an `[ENT-3]` source of its own, and does `[MSR-3]`'s deferred multi-payload restriction lift? | Yes to both; Rule 2's payload step is unusable without them. |
| 33 | Does "pairwise" in Rule 13 mean every pair in a run, or only adjacent pairs? | Every pair, the conservative reading `[PAR-1]` already takes. |
| 34 | Does a statement's own argument evaluation count in the overlap comparison, and is a `let`'s defined binding a write path? | Keep both `[PAR-1]` clauses as written; x1 mentions neither. |
| 35 | Does `[PAR-2]`'s refusal of `return`, `give`, `break` and `propagate` in a loop body survive, now that `[PAR-1]`'s exit-edge denial is deleted by Rule 14's overlapping `?` example? | Keep the loop-body refusal; but note the two rules then answer the same question differently. |
| 36 | Can a `Ring` carry a `[PAR-2]` element map at all, given a wrapping logical index is not a linear physical offset? | Restrict the element family to `Array` and `Slots` and say so. |

### Prelude, entry and housekeeping

| # | Question | Recommendation |
|---|---|---|
| 37 | What is `Oom`, and does Rule 5's `(Oom, Node)` payload require a tuple type, which `[TYPE-2]` does not have? | Declare `Oom` in `[PRE-1]`; prefer a two-field error struct over adding tuples in this amendment. |
| 38 | Does the entry keep the `Inputs` struct, with the heap ambient and unspellable? | Yes; delete only the `Heap<'s>` parameter sentence of `[PROG-3]`. |
| 39 | Does `if let` exist, or is it pseudocode for the existing exhaustive match? | Pseudocode; `[ERR-2]` stays as written. |
| 40 | Is the `..` rest binder admitted only in a destructuring consume, never in a match arm? | Yes; `[ERR-2]`'s no-wildcard rule is untouched. |
| 41 | Do the three cases that lose their subject retire with an honest note each: `accept-sysentry-command-all-inputs`, `fn4-neg-pure-member-binds-release`, `par3-pos-a-per-iteration-run-from-the-store-is-iteration-own`? | Yes, individually, not folded into a bulk rewrite. |
| 42 | May the checker's disjointness facts be handed to the backend as `noalias`, `captures(none)`, `memory(argmem)`, scoped alias metadata and loop parallel accesses, given `[DIAG-2]`'s broad ban on optimizer facts? | Yes, with an explicit permission stating that it adds no runtime branch, lock, dependency or scheduling edge. |
| 43 | `[S39]` and `[S34]` are cited (nine and several times) but defined nowhere in the active file. Inline the surviving content or drop the citations, and does `make check` gain a cited-tag-is-defined check? | Drop both with their subjects; add the check, since `[META-1]` and `[META-4]` already promise it. |
| 44 | Version numbering: the active file's title says v0.59 while the task text says v0.57. Does the amendment land as v0.60? | Yes, unless the owner says otherwise. |
| 45 | Does `[OWN-8]`'s reject-when-unsure disposition stand, given Rule 16 deliberately admits a stale but in-bounds handle as a logic error? | Yes; Rule 16 changes what is safe, not what an unproved judgment does. |
| 46 | Does the retirement of `dispose` leave no early-release form at all, accepting the peak-resource cost? | The owner must accept it or ask for a checked early release in a later round; x1 offers no replacement. |
| 47 | Does `[OWN-9]`'s non-normative optimizer paragraph survive, and what may it assert with no `&uniq`? | Restate it over declared write paths and proved disjointness, or drop it. |
| 48 | The candidate text contains mangled spellings (`writesderef(v)`, `entryderef(b).len`, `readsderef(v)`) where `writes(deref(v))`, `entry(deref(b)).len` and `reads(deref(v))` are meant. Confirm before they are copied into normative rows. | Confirm; they are plainly a stripped dereference marker. |

### Design-tree placement

| # | Question | Recommendation |
|---|---|---|
| 49 | Rule 16 reverses `design/language/ownership/affine-replacement.md`, which the live tree still holds. Does that ruling land in the same revision as the specification change? | Yes; otherwise `[STOR-1]`'s deleted pool rejection leaves spec and tree in conflict. |
| 50 | Three x1 decisions are drafted twice by different readers: Rule 1's closure (`data-model/value-semantics` versus `ownership/no-stored-references`), Rule 16 (`data-model` versus `ownership/pools-and-arenas`), and the overlap judgment (`effects/call-site-check` versus `ownership` root and `parallelism`). Which node owns each? | One node each; recommend data model for Rule 1, ownership for Rule 16, effects for the overlap judgment, with the others citing it. |
| 51 | Who owns Rule 14 (one heap, allocation carries no entry, no store parameters) in the design tree? No drafted node claims it. | Data model or system interface; several drafted nodes already stand on it. |
| 52 | Should `design/language/ownership/multi-target-commit` be renamed, since after the replacement it decides a two-place exchange? | Rename to `language/ownership/exchange`; that is itself a tree change the owner rules on. |
| 53 | Amendment file form: `design/skill/lint.py` reports prose lines outside the node template, yet the skill requires an amendment to name the decisions it replaces. Which convention wins? | Encode retirements as `Rejected:` items, which passes the lint; several drafted files currently fail it. |

---

## 5. Design amendments drafted

Thirty-two files in `amendments/`, one per node.

| File | Node | What changes |
|---|---|---|
| `design-language-ownership.md` | language/ownership | Borrow modes, regions and enumerated loan endpoints are replaced by a reference as a name for a path plus validity as a fact; overlap is generalized to serve validity, calls and overlap alike; the refusal of a kernel swap is retired. |
| `design-language-ownership-affine-replacement.md` | language/ownership/affine-replacement | `replace` becomes an operation on a reference rather than a let-only statement; the slice and arena carve-outs retire; whole-owner consuming moves, the element-move refusal and the atomic in-place update are added. |
| `design-language-ownership-copy-classification.md` | language/ownership/copy-classification | Copy and affine survive as type-fixed classes, but the carriers change: references are not values and slice views do not exist; linear joins as the third class. |
| `design-language-ownership-linearity.md` | language/ownership/linearity | Capability-relative linearity is replaced by type-declared linearity propagating through every aggregate; `dispose` retires; the zero-length release exemption becomes `free_empty` with a proved-empty contract. |
| `design-language-ownership-multi-target-commit.md` | language/ownership/multi-target-commit | The multi-place commit is replaced by a built-in two-place exchange admitted even when both references name the same place. |
| `design-language-ownership-no-reborrow.md` | language/ownership/no-reborrow | Node deleted: rebinding is admitted by Rule 2 and the whole reborrow, holder and region-endpoint family has no subject; its surviving question moves to reference-rebinding. |
| `design-language-ownership-region-elision.md` | language/ownership/region-elision | Node deleted: both decisions are region-spelling rules and there are no regions. |
| `design-language-ownership-slice-result-provenance.md` | language/ownership/slice-result-provenance | Node deleted: a range is a path, not a value with origins, and no reference is returned; successors are range-reference and no-stored-references. |
| `design-language-ownership-reference-validity.md` | language/ownership/reference-validity | New node: the path form, the payload step under a refinement fact, index capture at formation, validity as a fact with its invalidation events, and the rule that a move never re-roots. |
| `design-language-ownership-reference-rebinding.md` | language/ownership/reference-rebinding | New node: rebinding admitted, the join target set, and the static path shape that refuses a loop-carried self-extension. |
| `design-language-ownership-range-reference.md` | language/ownership/range-reference | New node: `&x[lo..hi]` with its formation obligation, the `&[T]` parameter kind, re-slicing, and how two ranges are separated. |
| `design-language-ownership-no-stored-references.md` | language/ownership/no-stored-references | New node: no aggregate, type argument, wrapper or payload holds a reference, and no reference is stored, returned or captured. |
| `design-language-ownership-pools-and-arenas.md` | language/ownership/pools-and-arenas | New node: a pool is a window plus indices, an arena is the same storage with append as allocation, and a stale in-bounds handle is a logic error. |
| `design-language-effects.md` | language/effects | Effects are declared only on reference parameters, by-value parameters carry no entry, the store-branded allocation decision retires, and the fact-kill decision is restated over declared writes. |
| `design-language-effects-call-site-check.md` | language/effects/call-site-check | New node: substitute actual paths, compare pairwise, reject an unproved overlapping pair with a write, count a by-value argument, and use one overlap judgment for all three consumers. |
| `design-language-contracts.md` | language/contracts | Structural contract equality at a supplied function is replaced by refinement (row subset, weaker `requires`, stronger `ensures`); the region and store-brand clauses retire. |
| `design-language-generics.md` | language/generics | Monomorphization and the cycle rule are kept unchanged; only their region vocabulary goes. |
| `design-language-data-model.md` | language/data-model | Array-of-structs moves onto the storage shapes, stable identity stops being a type axis, regions leave type identity, `Box` becomes the unbranded cell with a payload hand-back, and exclusivity on a mutation helper becomes a declared row plus the call-site check. |
| `design-language-data-model-kernel-minimality.md` | language/data-model/kernel-minimality | The kernel boundary moves: exchange, insertion and middle removal enter the kernel because no source body can write them; growth, pools, arenas, tables and strings stay in the library. |
| `design-language-data-model-storage-shapes.md` | language/data-model/storage-shapes | New node: three shapes with two placements, measures as read-only pseudo-fields, named window parts as row and overlap vocabulary, and how a value leaves a slot. |
| `design-language-data-model-value-semantics.md` | language/data-model/value-semantics | New node: every aggregate holds only owned values, recursively and closed under wrapping, so any value relocates by byte copy. |
| `design-language-checks-and-proofs.md` | language/checks-and-proofs | The fixed-family and certificate decisions are kept; two decisions are added, the absence of quantified and per-slot facts, and that disjointness is discharged by the same fixed families with an undischarged pair counting as overlapping. |
| `design-language-parallelism.md` | language/parallelism | Permission stays derived and failure stays non-rejecting; the staged-loop refusal is restated over declared rows instead of retained loans. |
| `design-language-parallelism-permission-judgment.md` | language/parallelism/permission-judgment | The window judgment is replaced by adjacency plus pairwise composition over paths; the state of interpretation and the allocation-is-not-an-effect clause are added. |
| `design-language-parallelism-loop-permission.md` | language/parallelism/loop-permission | The admitted loop forms are unchanged, but a partition becomes a range reference passed as an ordinary argument and independence becomes ordinary range disjointness. |
| `design-language-surface-form-borrow-lexicon.md` | language/surface-form/borrow-lexicon | `&uniq` is replaced by an unmarked reference whose write authority is its row; `&[T]` is named as the range-reference spelling. |
| `design-language-surface-form-construction-form.md` | language/surface-form/construction-form | Named construction is kept and a rest marker is added for the destructuring consume, with its linear-field restriction. |
| `design-language-surface-form-measure-spelling.md` | language/surface-form/measure-spelling | New node: measures are written as read-only members, and the four window parts are member names used only by rows and the overlap judgment. |
| `design-language-surface-form-in-place-update-form.md` | language/surface-form/in-place-update-form | New node: the atomic update is written as an assignment whose right-hand side is a call taking the same place by value. |
| `design-language-system-interface.md` | language/system-interface | The separately branded `Heap` parameter goes with Rule 14; `Inputs`, linear resource owners and explicit close survive unchanged. |
| `design-language-system-interface-handle-factory.md` | language/system-interface/handle-factory | Whole-call exclusivity on the factory becomes the call-site pairwise comparison of substituted rows; the credit accounting is unchanged. |
| `design-language-system-interface-directory-enumeration.md` | language/system-interface/directory-enumeration | The batch destination becomes a caller-formed range reference, the borrowed-entry alternative is now refused by the language itself, and the ground for separate numeric results is restated without loans. |

---

## 6. Two notes on the amendment as a change

Version. The active file's own title line says v0.59 while the task text says v0.57; on the
repository's rule the amendment lands as one change that retitles the active file vN+1 and archives
the outgoing bytes as `spec/kernel-spec-vN.md`.

Machine checks. `runner.py` fails on a bracketed reference that does not resolve and on an active
rule with no coverage, so within the single change every deleted tag must lose every citation and
every case or annotation, and every added tag must gain one. That is why step 12 exists and why
`[S39]` and `[S34]` cannot simply be left where they are.

## 7. Triage of the open questions

The 53 questions above were raised by the section readers. Numbers refer to that list.

### Decided by candidate x1 or by an earlier owner ruling; the implementer applies them

| Question | Disposition |
|---|---|
| 2 | `deref` is only the Box-content step; a reference is a name and is never dereferenced. |
| 6 | `&[T]` is a parameter-only reference kind, never a type in value position. |
| 8, 10, 31 | `len`, `cap`, `room`, `head`, `next`, `last`, `filled`, `free` are reserved pseudo-fields; `Array`, `Slots`, `Ring`, `Box` are nominals; measures are place forms, not reader rows. |
| 14, 15, 17 | `replace` stays general over owned places and stays rejected over copy places; `truncate` is library code. |
| 20, 21, 22, 23 | A row states exactly what the body does; for one parameter `writes` implies the read of the same path and never self-conflicts (Rule 10 compares different arguments); a category may repeat; no both-categories clause. |
| 24 | A body cannot move out through a reference parameter: Rule 6 admits moves only by consuming a whole local, the window operations, and the atomic update, and a callee owns no local for the caller's value. |
| 33, 34, 35 | Overlap permission is every pair in a run; argument evaluation and `let` bindings count as today; the loop-body exit refusals stay. |
| 36 | The counted-loop element-map family is restricted to `Array` and `Slots`; a `Ring` index wraps. |
| 39, 40 | `if let` was pseudocode for the exhaustive match; `..` appears only in a consuming destructuring. |
| 45, 49 | Reject-when-unsure stands; the tree ruling reversing the pool rejection lands with this revision. |
| 43, 48 | Undefined tags go with their subjects; the candidate's mangled `writesderef` spellings were a transcription defect and are fixed. |
| 25, 26, 30, 50, 51, 52, 53 | Placement and ownership: the call-site rule in section 9 cited from section 5; window parts and measures in storage, the no-entry half of the allocator axiom in effects; validity is an ownership judgment outside the entailment fragment; Rule 1's closure is owned by data-model, Rule 16 by ownership, the overlap judgment by effects, Rule 14 by data-model; `multi-target-commit` becomes `exchange`; retirements are encoded as `Rejected:` items. |
| 13, 18, 19, 32, 38, 41, 47 | As the readers recommend: `grow` re-runs the layout ceiling; capacity-boundary refusals stay; the runtime-length `Array` carries its length word and the no-stored-length rule narrows to constant `Array`; a match arm's refinement fact has an entailment source; `Inputs` stays with the heap ambient; the three subject-less cases retire individually; the optimizer paragraph is restated over declared writes and proved disjointness. |
| 29 | `compare_op` already admits `==`; the invariant example stands. |

### Owner rulings needed before the corresponding step

| Question | Choice | Recommendation |
|---|---|---|
| 1 | Keep `pure` as the spelling of the empty row | Keep. |
| 3, 4 | Retire the `let x = replace p = e;` statement and the multi-target `set (a, b) = ...` in favour of the built-in `replace` and `swap` | Retire both; a three-way rotation is two `swap`s. |
| 9, 11 | Constructor spelling and array literals | Prelude functions in today's snake-case style (`box_new`, `slots_new`, `array_filled`, `box_slots_new`), no `Type::name` form; no `[a, b, c]` literal in this revision. |
| 12 | Static allocation-fit obligation versus the runtime `Result` | Runtime `Result` only (Rule 14); the AllocationFit family and its diagnostics retire. |
| 16 | A range reference into a `Ring` | Refuse; a ring hands out single slots only. |
| 27 | May the empty row license deduplicating an allocating call | No; allocation failure is observable. |
| 28 | How a supplied function's weaker `requires` and stronger `ensures` are checked | A fixed finite check inside the existing affine entailment fragment, no solver. |
| 37 | The failure payload of a fallible allocation, given the kernel has no tuples | A prelude struct `AllocationFailed<T> { value: T }` returned in the `Err` arm; no tuples added. |
| 42 | Handing the checker's facts to the backend against the diagnostics rule that bans optimizer facts | Add an explicit permission: facts may be emitted as attributes and metadata that add no runtime branch, lock, dependency, or scheduling edge. |
| 44 | Version number | The amendment lands as v0.60 over v0.59. |
| 46 | No `dispose` leaves no early release for an affine value | Add a prelude `release(move x)` consuming any affine value; zero cost. |
