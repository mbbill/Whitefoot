# Conformance corpus classification: kernel specification v0.59 -> v0.60 (candidate x1)

Inputs read: `/private/tmp/whitefoot-spec-x1/tests/conformance/runner.py`, `/private/tmp/whitefoot-spec-x1/tests/conformance/manifest.jsonl` (1133 case rows + 16 `covered_by` annotations), the 1133 sources under `/private/tmp/whitefoot-spec-x1/tests/conformance/cases/`, `/private/tmp/whitefoot-spec-x1/spec/kernel-spec.md` (v0.60), `/private/tmp/whitefoot-spec-x1/spec/kernel-spec-v0.59.md`, and `<session scratch>`.

## What the manifest requires

`runner.py` enforces four things that decide this classification (`validate_manifest`, lines 236-330;
`coverage`, lines 332-351; `spec_rule_ids`, lines 196-204):

1. **Every cited rule must be defined in the active spec.** `validate_manifest` collects the rule ids
   defined at line starts of `spec/kernel-spec.md` and rejects any case whose `rules` list names one that
   is absent (`unknown rules`, line 262). A `[FAM-N.Sk]` citation folds onto its parent (`base_rule`,
   line 191). So every case citing a retired tag is a hard manifest error the moment v0.60 is the active
   file, whatever the case source says.
2. **Every active rule must be covered.** `coverage` computes `rules - covered`, `main` counts a nonempty
   uncovered set as a failure (lines 478-483). Coverage counts a rule as covered when some case tags it or
   some `covered_by` annotation names it. The denominator moves from 129 to 125 base rules.
3. **Verdict kinds are a closed set**: `accept`, `reject` (with `rule`, which must also appear in the
   case's `rules`), `run` (with an integer `exit`), `unsupported` (with `why`). `status` is the separate
   readiness axis (`runnable` / `pending` / `xfail`) and never rewrites `expect`.
4. **Structural integrity**: case ids unique, one `<id>.wf` per row and no orphan sources, `arrange` only
   on a `run` case and only with the closed key set, every case carries a `doc`.

The manifest is 539 `reject`, 356 `run`, 238 `accept`; 1131 `runnable`, 1 `pending`, 1 `xfail`.

## Method

Verified by reading: `runner.py` in full; the v0.60 rules TYPE-7, REF-1..4, EFF-1, EFF-2, EFF-5, OWN-1,
FORM-3, GRAM-4, PRE-1, MSR-4 and the FN-8 diagnostic paragraph; the tag ledger's five sections; the
`doc` line of every case whose expected rejection rule is retired and of every case whose whole cited rule
set is retired; and roughly thirty case sources in full.

Verified mechanically over all 1133 sources (with `doc "..."` strings stripped first, so prose never
triggers a hit): presence of each retired construct, citation of each retired tag, `reads`/`writes` rooted
at an `own` parameter (a per-signature parameter-mode scan), declaration of one of the eight names FORM-3
newly reserves, and `==` in an invariant target.

Inferred, not verified: that a rewrite of a surviving rule leaves a case that uses none of the retired
vocabulary alone. I checked this for the whole `unchanged` set by two sweeps — every identifier used in a
callee or type-application position resolves in v0.60, and no word that exists in v0.59 but not in v0.60
appears outside a local binder — but I did not re-derive each case's verdict against the v0.60 text.

## Retired tag -> successor citation

23 tags leave the specification. A surviving case that cited one must re-cite:

| retired | fate in ledger | successor citation for a surviving case |
|---|---|---|
| FORM-8 | delete | none — regions do not exist |
| OWN-2 | replace | GRAM-3 (`mode := "own" \| "&"`) and REF-1 |
| OWN-3, OWN-4, OWN-10 | delete | none — lexical regions and region-based borrow liveness are gone |
| OWN-5 | replace | REF-2 (validity) and EFF-5 (the call-site pairwise check) |
| OWN-6 | replace | REF-1 and REF-2 |
| OWN-12 | replace | EFF-5 |
| OWN-14 | delete | none — REF-3 forbids returning a reference at all |
| LIV-2 | replace | SET-1 (one commit) and OP-11 (`swap`) |
| SET-2 | replace | OP-11 and OP-12 |
| PROV-1 | delete | none — store identity was a region |
| BLK-0 | delete | FN-8 for an undischarged `requires` at an ordinary PRE-1 call |
| BLK-1 | replace | TYPE-9 (the shapes) and WIN-1 (the window) |
| BLK-2 | delete | OP-13 and the PRE-1 constructor records |
| BLK-3 | replace | OP-10 and the PRE-1 window-operation records |
| BLK-4 | delete | none — confinement and position closure are gone |
| VIEW-1, VIEW-4, VIEW-6 | delete | none — view values are gone |
| VIEW-2 | replace | REF-4 |
| STOR-2 | replace | OP-13 and the PRE-1 constructor records |
| STOR-4 | delete | none — arenas are gone |

## Mechanical rewrite key

The per-case reasons below name these token classes; each is one edit:


- **uniq** — `&uniq` borrow mode -> plain `&` plus `writes(path)` in the callee row [EFF-1]
- **region** — `region` / `region 'r` statement -> deleted; flatten the block into its enclosing statement list [GRAM-4]
- **regionparam** — `['r]` region parameter list -> deleted from every declaration [GRAM-2]
- **reflifetime** — `&'r T` -> `&T` [GRAM-3]
- **borrowresult** — borrow-mode result -> deleted; `rtype := "own" type`, return an index or owned data [GRAM-3, REF-3]
- **deref** — `deref(r)` on a reference -> read the reference bare; `deref` is the `Box`-content step alone [TYPE-7]
- **measure_fn** — `len_of/cap_of/room_of/head_of(p)` -> the pseudo-field places `p.len`, `p.cap`, `p.room`, `p.head` [OP-15, MSR-1]
- **vector** — `FixedVector`/`Vector`/`fixed_vector`/`heap_vector`/`arena_vector*` -> `Slots<T, n>` / `Box<Slots<T>>` via `slots_new` / `box_slots_new` [TYPE-9, OP-13]
- **slice** — `Slice`/`MutSlice`/`slice_of`/`mut_slice_of` -> the range reference `&x[lo..hi]`, parameter kind `&[T]` [REF-4, TYPE-8]
- **arena** — arenas, `arena_frame`, `arena_new`, `Arena<'s,..>` -> deleted; one heap, total allocation [STOR-8]
- **heap** — `Heap<'s>` / `heap_*` providers -> deleted; there is one heap and no store value [STOR-8, PROV-1 retired]
- **store_arg** — store/region type argument in `::<..>` -> deleted [GRAM-3]
- **lifetime** — region argument in a type application -> deleted [GRAM-3]
- **boxstore** — `Box<'s, T>` -> `Box<T>` [TYPE-9]
- **array_old** — `array<T, n>` / `array_new` -> `Array<T, n>` / `array_filled` [TYPE-9, OP-13]
- **buffer** — `buffer<u8>` -> `Slots<u8, n>` or `Box<Slots<u8>>` [TYPE-9]
- **allocates** — `allocates` effect category -> deleted; allocation contributes no effect path [EFF-2, STOR-8]
- **owneffect** — `reads`/`writes` rooted at an `own` parameter -> deleted; every effect path is rooted at a reference parameter [EFF-1]
- **replace** — `replace` statement -> `swap(p, q)` [OP-11] or `set p = f(p, ...)` [OP-12]
- **multiset** — multi-target `set a, b = ..` -> deleted; one commit per statement [SET-1], exchange via `swap` [OP-11]
- **dispose** — `dispose` statement -> deleted; release is compiler-derived at scope exit [STOR-3, PROV-6]
- **fits** — `fits_element`/`buffer_fits`/`buffer_vacant` -> the `requires` contracts of the [PRE-1] constructors [OP-13]

## Totals

| fate | cases | share |
|---|---|---|
| unchanged | 416 | 36.7% |
| rewrite | 602 | 53.1% |
| retire | 115 | 10.2% |
| **total** | **1133** | 100% |

| new cases needed | 19 |
|---|---|

## Rules no surviving case covers (new_needed)

All 19 are the tags v0.60 mints; none of the 1018 surviving cases can cite a rule that did not exist when it was written. Eight of them (REF-1, REF-2, REF-3, REF-4, EFF-5, TYPE-9, OP-13, STOR-8) are picked up by the re-citations the rewrite rows below name, so they need new cases only for the parts those rewrites do not reach; the other eleven have no candidate case at all and need original ones.

| rule | needs original cases | what is uncovered |
|---|---|---|
| EFF-5 | partly (a rewrite re-cites it) | the call-site substitution and pairwise overlap check, and the post-call invalidation of bystander references |
| OP-10 | yes | the ten window operations, their rows over the window parts and `len`, and the compiler-owned window type parameter |
| OP-11 | yes | `swap(p, q)` and its same-place exception |
| OP-12 | yes | the atomic in-place update `set p = f(p, args...)` |
| OP-13 | partly (a rewrite re-cites it) | the construction set and the static size-overflow obligation on runtime-capacity constructors and `grow` |
| OP-14 | yes | `free_empty(move r)` under `requires r.len == 0` |
| OP-15 | yes | a measure read is a place form, not a call, and is never assignable |
| REF-1 | partly (a rewrite re-cites it) | a reference is a local name for a path; index evaluated at formation; rebinding; the join path set; static path shape |
| REF-2 | partly (a rewrite re-cites it) | reference validity as a checked fact and its exact invalidation events; use of an invalid reference |
| REF-3 | partly (a rewrite re-cites it) | references never escape: not into an aggregate, not returned, not captured by a stored function value |
| REF-4 | partly (a rewrite re-cites it) | range references `&x[lo..hi]`, the `&[T]` parameter kind, re-slicing, and the `Ring` refusal |
| STOR-7 | yes | any value may be relocated by copying its bytes; no address is depended on |
| STOR-8 | partly (a rewrite re-cites it) | one heap, total allocation, exhaustion outside the language, and the `no_heap` program declaration |
| TYPE-10 | yes | `len`/`cap`/`room`/`head` are read-only pseudo-fields; `next`/`last`/`filled`/`free` occupy no declaration domain and no program reads them |
| TYPE-8 | yes | a reference kind is not a value type; no aggregate, type argument, wrapper or variant payload may hold one |
| TYPE-9 | partly (a rewrite re-cites it) | the three storage shapes `Array`/`Slots`/`Ring` in constant- and runtime-capacity form, and `Box<T>` |
| WIN-1 | yes | the window: values below `r.len`, nothing above; no tags; `a.len == a.cap` for `Array` |
| WIN-2 | yes | the four window parts as effect-row/overlap vocabulary with their fixed overlap answers |
| WIN-3 | yes | how a value leaves storage: whole-owner consume, `..` rest, refusal of a move out of a window slot, release on overwrite |

Beyond the rule denominator, two coverage holes the port opens and that `runner.py` cannot see,
because `coverage` counts tags and not subjects:

- **INV-1 `==`.** `inv1-neg-equality-relation` is retired because the ledger's section 4(a) decision makes
  `==` admitted in a `header_invariant` and an `invariant_stmt` target. INV-1 stays covered by 39 other
  cases, but the new acceptance has no positive and the surviving refusals (`!=` anywhere, `==` in a
  `use_premise`) have no negative.
- **REF-1 index capture.** `own5-neg-captured-index-loan` and `own5-neg-captured-range-endpoint-reassignment`
  are retired because REF-1 now *accepts* what they refuse (the index is evaluated when the reference is
  formed). Both should return as positives.

## One line per case

Format: `id | fate | reason`. Sorted unchanged, then rewrite, then retire; alphabetical within each.

### unchanged (416)

```
accept-sysentry-command-no-inputs | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
accept-sysname-near-lookalike | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
const2-neg-enum-not-const-eligible | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
const2-neg-set-target | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
const2-pos-item | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
const2-pos-struct-const | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
const2-pos-struct-field-labels | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
eff1-neg-unused-formal-formation-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
eff2-pos-discharged-division-site-pure-row | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
eff3-pos-pure-fn | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent3-neg-runtime-division-dividend-replaced | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent3-neg-runtime-division-divisor-replaced | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent3-neg-runtime-division-quotient-replaced | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent3-neg-stage8b-local-one | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent3-pos-stage8b-bit-sources | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-neg-replacement-overflow-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-neg-value-if-unbounded-delivery | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-neg-value-match-no-delivery | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-pos-close-before-set-strengthening | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-pos-project-middle-before-set | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-pos-quotient-alias-after-set | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent5-pos-value-if-delivery-join | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent6-neg-nested-join-does-not-invent-a-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent6-pos-join-shape-independence | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
ent6-pos-nested-join-keeps-the-flat-image | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err1-pos-aggregate-result-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err1-pos-result-value-match | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err2-neg-missing-variant | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err2-pos-exhaustive-match | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err3-neg-error-type-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err3-neg-propagate-different-error-type | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err3-pos-propagate | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err3-pos-propagation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err4-checked-domain-violation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
err4-pos-recoverable-value | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-neg-equal-width-unsigned-to-signed-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-neg-narrowing-result-as-scalar | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-neg-signed-to-wider-unsigned-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-pos-generic-main | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-pos-library-without-entry | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn1-pos-signature-driven-call | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-neg-eeq-implicit-type | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-neg-qualified-member-specialization-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-neg-trailing-type-argument | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-neg-wrong-kind-instantiation-argument | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-pos-qualified-member-specialization-2 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn2-pos-the-template-is-the-spelling-authority | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-bare-type-parameter-has-no-formal-group | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-contract-arguments | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-extra-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-formal-generic-header-2 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-missing-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-out-of-order-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-neg-source-contract-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-contract-conform | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-distinct-actual-names | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-empty-marker-conformance | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-formal-generic-header-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-generic-formal | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn3-pos-two-explicit-actuals | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn4-neg-function-signature-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn4-neg-requires-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn4-neg-structural-ensures-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn5-neg-ambiguous-formal-member | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn5-pos-match-dispatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn6-neg-growing-function-argument | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn6-neg-polymorphic-recursion | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn6-pos-recursion | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn7-neg-missing-command-marker | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn7-neg-two-mains | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn7-pos-single-main | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-constant-call-requirement | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-empty-contract | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-entry-contract | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-control | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-eeq-integer | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-local-in-body | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-move-operand | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-no-check | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-non-bool-check | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-noncopy-cvt-local | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-partial-op | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-set | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-neg-requires-user-call | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-pos-bounded-sum-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-pos-requires-eeq | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-pos-requires-name-reuse | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn8-pos-requires-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-division-floor-equality | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-entry-image-kill | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-named-outcome-no-publication | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-no-selected-normal-exit | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-same-scc-summary | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-saturation-is-not-wrapping | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-strict-counted-exhaustion-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-unproved-selected-return | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-widening-is-not-reinterpretation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-neg-wrapped-negation-nonnegative | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-pos-counted-exhaustion-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-pos-direct-set-receiver | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-pos-ok-selected-receiver | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-pos-plain-direct-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
fn9-pos-two-signed-result-bounds | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form1-neg-unknown-construct | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form1-pos-canonical | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form2-neg-noncanonical-ws | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form2-pos-canonical-bytes | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form3-neg-opname-bad-suffix | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form3-neg-requires-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form3-neg-retired-mode-suffix | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form3-neg-typeid-fn-name | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form4-neg-comment | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form4-pos-doc-not-comment | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form5-neg-missing-suffix | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form5-pos-suffixed-literal | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form6-pos-unit-type-and-value | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form7-neg-leading-zero | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form7-neg-out-of-range | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
form7-pos-in-range | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram1-pos-lookahead | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram10-neg-wrong-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram10-pos-named-binders | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram11-neg-misspelled | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram11-neg-out-of-order | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram11-pos-named-args | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram2-neg-retired-behavior-law | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram2-pos-items | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram4-neg-bare-relation-premise | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram4-neg-multiplied-use-relation-bare | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram4-neg-typed-multiplicity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram4-pos-nested-break-to-counted-label | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram4-pos-nested-ordinary-label-break | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram5-neg-type-application-without-delimiter | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram5-pos-comparison-operators | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram6-pos-conditionals-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram6-pos-no-operators | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram7-pos-two-productions | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram8-neg-constructor-call-boundary-4 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram8-neg-constructor-call-boundary-5 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram8-neg-out-of-order | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram8-neg-wrong-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram8-pos-construct | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram9-neg-constructor-in-call-argument | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram9-neg-constructor-in-constructor-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram9-neg-nested-call | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
gram9-pos-three-address | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-an-affine-atom-is-not-a-bare-local | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-backedge-unproved | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-base-unproved | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-division-operand-replacement-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-division-operand-replacement-2 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-division-operand-replacement-3 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-division-operand-replacement-4 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-forward-proof-name | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-header-name-after-loop | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-header-refers-to-body-local | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-local-relation-unproved | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-ordinary-cursor-overshoot | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-repeated-name | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-replaced-loop-limit | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-neg-sequential-guarded-steps | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-automatic-two-premise-backedge | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-counted-accumulator-bounds | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-grouping-node-count | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-guarded-variable-step | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-index-weighted-accumulator | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-labeled-loop-simultaneous-headers | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-ordinary-loop-cursor-set-image | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-ordinary-loop-guarded-cursor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-remainder-restores-header | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
inv1-pos-signed-descending-accumulator | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
msr3-neg-a-parameter-value-written-back-loses-its-entry-image | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
msr5-neg-a-clause-side-is-not-a-difference-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
msr6-pos-const-generic-as-a-value | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-eeq-arity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-eeq-distinct-enums | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-eeq-integer | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-eeq-payload-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ene-integer | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-enum-ordering | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ieq-bool | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ieq-tag-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ine-bool | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ine-tag-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-ineg-unsigned | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-neg-written-argument-on-deargumented-row | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-bool-enum-equality | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-expression-positions | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-infix-return-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-infix-rows-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-integer-width-sign-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-operand-selected-minimum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-table-op | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op1-pos-tag-enum-equality | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-branch-quotient-images | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-break-removes-exhaustion-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-byte-count-past-maximum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-combined-exhaustion-overflow | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-counted-accumulator-without-invariant | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-counted-subtraction-underflow | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-div-wrap | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-division-constant-zero-divisor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-division-minus-one-divisor-unbounded | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-division-obligation-undischarged | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-exact-absolute-minimum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-exact-negation-minimum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-insufficient-multiply-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-left-shift-amount-unbounded | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-nonzero-signed-divisor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-overflow-obligation-undischarged | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-quotient-replacement | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-reversed-unsigned-subtraction | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-right-shift-amount-unbounded | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-runtime-division-product-domain | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-signed-decrement-minimum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-signed-order-does-not-bound-difference | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-unguarded-signed-negation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-unproved-signed-variable-division | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-neg-wrong-remainder-guard | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-bounded-arithmetic-images | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-div-checked | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-checked-untouched | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-constant-divisor-total | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-minus-one-divisor-bounded | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-obligation-discharged | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-remainder-same-obligation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-division-requires-nonzero | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-early-return-divisor-guard | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-generic-division-domain | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-guarded-subtraction-l0-bridge | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-ineg-modes | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-overflow-obligation-discharged | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-runtime-division-aliases | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-runtime-division-images | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-sat-mode | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-signed-domain-sources | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-unsigned-literal-division-images | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op2-pos-wrap-untouched-by-dissolution | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op3-neg-exact-dotted | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op3-pos-fadd-strict | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op5-neg-unused-formal-formation-3 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op6-neg-cvt-identity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op6-pos-cvt-checked | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op6-pos-cvt-total | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op6-pos-inferred-float-conversion-result | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op6-pos-signed-conversion-images | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op7-neg-missing-prefix | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op7-pos-name-convention | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op8-neg-rotate-checked | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op8-pos-iand | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op8-pos-integer-family | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
op8-pos-u64-shift-u32 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-neg-match-move-copy | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-neg-move-of-copy | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-neg-use-after-move | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-pos-a-copy-bounded-body-duplicates-its-value | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-pos-consume-once | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-pos-match-copy-payload-reuse | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-pos-propagate-copy-payload-reuse | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own1-pos-tagonly-copy | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own11-neg-move-outer-in-loop | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
own13-pos-let-match-give | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
pending-result-aggregate-payload | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
pre1-pos-affine-opaque-empty-drop | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
pre1-pos-prelude-enums | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-certified-image-after-replacement | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-duplicate-named-and-relation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-duplicate-scaled-use | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-duplicate-use | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-explicit-factor-one | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-fact-killed-by-write | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-inexact-combination | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-literal-factor-zero | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-mismatched-coefficient-vector | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-redundant-counted-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-redundant-mixed-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-redundant-pair-bound | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-redundant-use-block | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-signed-multiplicity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-unfolded-nonlinear-sum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-neg-unproved-premise | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-active-header-reference | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-binder-multiplicity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-certificate-after-exhaustion | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-chained-weighted-certificates | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-derived-multiplicity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-explicit-affine-proof | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-explicit-factor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-full-use-capacity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-mixed-named-and-relation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-multiplied-relation-use | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-proof-to-postcondition | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-reprove-replacement-image | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-term-multiplicity | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-term-multiplicity-shared-product | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-three-weighted-premises | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prf1-pos-write-after-certificate | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prog1-pos-closed-unit | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-a-copy-bound-refuses-an-affine-argument | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-a-linear-bounded-body-drops-its-value | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-linear-modifier-on-a-tag-only-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-linear-value-not-consumed | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-linearity-bound-refuses-the-instantiation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-neg-prior-ticket-leaked-on-refusal | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-pos-linear-value-moved-out-whole | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-pos-linearity-bound-admits-the-instantiation | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
prov6-pos-the-bound-chain-admits-every-lower-class | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-err2-nonexhaustive | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-form1-daemon-leading-construct | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-form1-embedded-leading-construct | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-form1-service-leading-construct | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-gram11-unnamed-call | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-syseff-return-unit-pure | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-call-to-kind-entry | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-input-type-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-label-out-of-order | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-label-outside-entry | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-label-repeated | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysentry-label-unknown | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysexit-return-u8-no-conversion | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-sysname-collision-in-kind-unit | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
reject-type5-system-arg-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
run-invariant-exact-sum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
run-sysexit-code-0 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
run-sysexit-code-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
run-sysexit-code-2 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
s7-pos-signed-remainder-ranges-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
scope1-pos-kernel-no-gated | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
scope2-pos-accepted | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
scope3-pos-defined-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
set1-neg-counted-binder-write | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
set1-pos-local-and-field-copy | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
stor1-neg-affine-set-target | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
stor1-pos-frame-resident | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
systcp-address-pure | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
systcp-connection-not-constructible | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type1-pos-i32-unit | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type1-pos-wide-prims | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type2-pos-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type2-pos-symbolic-prelude-instance | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type2-pos-twostate-enum-i1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type4-pos-cvt | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-arg-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-constructor-call-boundary-1 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-constructor-call-boundary-2 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-constructor-call-boundary-3 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-constructor-call-boundary-6 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-constructor-call-boundary-7 | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-match-non-enum | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-neg-mixed-sign-comparison | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type5-pos-unit-call-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-neg-dup-variant | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-neg-nested-live-label | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-neg-shadow | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-pos-counted-enclosing-break | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-pos-distinct-names | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
type6-pos-nested-break-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
v033-neg-exact-domain-unproved | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
v033-neg-missing-result-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-iadd-checked-overflow-err-arm-runs | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-iadd-wrap-overflow-to-negative | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-idiv-reject-constant-zero-divisor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-isub-wrap-min-roundtrip-runs | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-loop-checked-multiply-overflow | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-arith-wrapping-overflow-observed | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-const-scalar-u64-width | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-eff-writes-missing-region | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-multiwidth-dispatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-option-context-free-constructor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-payload-give | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-result-context-free-constructor | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-result-payload-type-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-stmt-payload-check | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-enum-twostate-result-payload | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-fn-arg-type-mismatch | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-fn-cross-fn-call-chain | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-fn-mutual-recursion-runs | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form2-tab-indent | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form3-enum-name-ident | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form4-block-comment | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form5-op-arg-missing-suffix | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form7-checked-overflow-canonical | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-form-form7-i32-max-plus-one | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-give-result-aggregate | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-call-repeated-arg | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-combo-flat-call-construct-match | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-construct-missing-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-construct-repeated-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-nested-op-in-construct-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-gram-nested-ucall-in-call-arg | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-integ-checked-overflow-diverts-to-err | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-integ-give-in-statement-match-rejected | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-integ-loop-product-checked-overflow | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-integ-sign-weight-accumulate | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-integ-traffic-light-state-machine | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-err2-value-match-missing-err | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-give1-give-in-stmt-match | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-give1-nested-value-match | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-give1-wrong-type | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-gram10-binder-not-fresh | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-match-gram10-out-of-order-fields | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-nominal-bool-ops-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-nominal-multifield-payload-run | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-ownmove-copy-reused-affine-consumed-once | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-ownmove-owned-temporary-scrutinee | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-ownmove-partial-move-kills-binding | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-ownmove-payload-binder-consumed-twice | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-struct-construct-read-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-struct-mixed-width | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-struct-neg-field-order | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-struct-nested-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-struct-set-field | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-typ-bool-cmp-result-as-int | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-typ-let-shadows-param | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
x-typ-match-foreign-variant | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
xfail-own1-bare-affine-use | unchanged | no retired syntax, no retired tag; the rewrites of the rules it cites do not touch it
```

### rewrite (602)

```
accept-par3-staged-denied-carried-scratch-byte | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-denied-exit-in-remainder | rewrite | retired syntax: deref, measure_fn, region, slice, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-denied-hoisted-scratch | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-denied-opaque-cursor | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-denied-read-before-write | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-iteration-own-scratch | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5
accept-par3-staged-loop-with-prologue-break | rewrite | retired syntax: deref, measure_fn, region, slice, uniq, vector; re-cite retired tags: OWN-5
accept-syseff-conditional-release-union | rewrite | retired syntax: deref, region, uniq
accept-syseff-pure-immutable-only | rewrite | retired syntax: owneffect, region
accept-sysentry-command-all-inputs | rewrite | retired syntax: heap, lifetime, region, regionparam, uniq
accept-sysfile-two-permits-shared-directory | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-5
accept-sysrelease-return-unit-declared | rewrite | retired syntax: deref, region, uniq
blk0-neg-a-proved-take-without-the-room-it-requires | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-neg-a-row-does-not-discharge-its-caller-s-obligations | rewrite | retired syntax: arena, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk0-neg-a-row-requirement-is-not-discharged | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-neg-a-source-function-collides-with-a-kernel-row | rewrite | re-cite retired tags: BLK-0
blk0-neg-a-view-over-a-wrapped-run | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-neg-full-array-freeze-requires-fullness | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-neg-the-none-arm-refutes-the-take-that-refused | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-neg-the-refusing-formation-row-discharges-nothing-either | rewrite | retired syntax: arena, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk0-neg-the-some-arm-s-advance-bounds-the-next-take | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
blk0-pos-a-routed-row-publishes-its-payload-on-the-some-arm | rewrite | retired syntax: arena, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk0-pos-a-row-s-published-state-is-what-its-caller-holds | rewrite | retired syntax: arena, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk0-pos-the-none-arm-knows-the-store-was-not-advanced | rewrite | retired syntax: arena, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk0-pos-the-some-arm-publishes-the-store-s-own-advance | rewrite | retired syntax: arena, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-0
blk1-neg-a-construct-names-a-run | rewrite | the refusal of a named construction of a compiler-owned storage nominal survives: re-cite TYPE-9 (with GRAM-8)
blk1-pos-a-run-element-is-a-run | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-1
blk1-pos-a-run-element-type-is-a-type-parameter | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-1
blk1-pos-a-wrapped-window-subscripts-against-len | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: BLK-1
blk1-pos-both-runs-are-nameable-types | rewrite | retired syntax: heap, lifetime, regionparam, vector; re-cite retired tags: BLK-1
blk2-pos-a-bump-take-hands-out-a-run | rewrite | retired syntax: arena, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-0, BLK-2, PROV-1
blk2-pos-a-formation-row-builds-an-empty-run | rewrite | retired syntax: vector; re-cite retired tags: BLK-0, BLK-2
blk2-pos-a-refused-take-leaves-the-store-unmoved | rewrite | retired syntax: arena, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-0, BLK-2
blk3-pos-a-boundary-row-moves-the-back-boundary | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-0, BLK-3
blk3-pos-a-deque-moves-both-boundaries | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-1, BLK-3
blk3-pos-a-queue-preserves-its-order | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: BLK-1, BLK-3
blk3-pos-full-array-owning-round-trip | rewrite | retired syntax: array_old, deref, measure_fn, owneffect, region, replace, uniq, vector; re-cite retired tags: BLK-0, BLK-3, OWN-5, SET-2
blk4-neg-a-field-branded-to-a-heap-the-unit-cannot-reach | rewrite | retired syntax: regionparam, vector; re-cite retired tags: BLK-4, PROV-1
blk4-pos-a-reference-holder-over-a-store-resident-run | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, uniq, vector; re-cite retired tags: BLK-1, BLK-4, OWN-5
blk4-pos-a-run-reached-through-a-shared-borrow-of-a-nominal | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-1, BLK-4, OWN-5
blk4-pos-a-shared-borrow-of-a-run-parameter | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-1, BLK-4, OWN-5
blk4-pos-a-unique-parameter-reaches-a-run | rewrite | retired syntax: deref, lifetime, measure_fn, regionparam, uniq, vector; re-cite retired tags: BLK-3, BLK-4
blk4-pos-a-unique-parameter-reaches-a-type-parameter | rewrite | retired syntax: uniq; re-cite retired tags: BLK-4
call1-pos-a-shared-borrow-keeps-every-fact | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: OWN-5
call2-neg-a-result-carries-only-the-contract | rewrite | retired syntax: region, uniq, vector
call2-pos-a-copy-actual-at-an-own-parameter-is-not-a-consume | rewrite | retired syntax: measure_fn, owneffect, region, slice, uniq, vector; re-cite retired tags: VIEW-1
call2-pos-an-own-operand-measure-reaches-the-result | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
call3-pos-a-fill-through-an-exclusive-view-keeps-both-lengths | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: VIEW-1, VIEW-2
call4-neg-ambiguous-route-over-two-enum-ordinals | rewrite | retired syntax: measure_fn, owneffect, slice
call4-neg-measured-result-not-admitted | rewrite | retired syntax: measure_fn, region, uniq, vector
call4-pos-omitted-route-binder-with-one-enum-ordinal | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
call4-pos-route-names-a-result-ordinal | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
call5-neg-a-bound-borrow-actual-kills-the-same | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
call5-neg-a-one-byte-body-kills-the-measure-the-same | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
call5-pos-view-write-measure-transport-1 | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector
call6-neg-contradictory-published-relations | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
call6-pos-a-row-relation-establishes-at-a-caller | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-0, BLK-3
call6-pos-routed-relation-over-a-call-datum | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
const1-neg-eval-overflow | rewrite | retired syntax: vector
const1-neg-noninteger | rewrite | retired syntax: vector
const1-neg-runtime-arithmetic-mode | rewrite | retired syntax: vector
const1-pos-array-size | rewrite | retired syntax: vector
const1-pos-forwarded-arithmetic | rewrite | retired syntax: measure_fn, uniq, vector
const2-neg-a-cell-is-not-const-eligible | rewrite | re-cite retired tags: PROV-1
const2-neg-a-fixed-vector-const-short-of-its-capacity | rewrite | retired syntax: vector
const2-neg-array-view-domain-7 | rewrite | retired syntax: array_old, region, slice, uniq; re-cite retired tags: OWN-5, VIEW-1
const2-neg-noneligible | rewrite | retired syntax: buffer
const2-neg-set | rewrite | retired syntax: vector
const2-pos-a-fixed-vector-const-run | rewrite | retired syntax: measure_fn, region, slice, vector; re-cite retired tags: VIEW-2
const2-pos-array-lookup | rewrite | retired syntax: measure_fn, vector
const2-pos-constant-nested-field-bound-1 | rewrite | retired syntax: array_old
const2-pos-float-storage-run | rewrite | retired syntax: region, uniq, vector
const2-pos-nested-array-borrow | rewrite | retired syntax: array_old, deref, region
eff1-neg-wrong-order-row | rewrite | retired syntax: allocates, deref, heap, lifetime, regionparam, uniq
eff2-neg-borrowed-column-write-effect | rewrite | retired syntax: buffer, deref, measure_fn, reflifetime, region, regionparam, uniq
eff2-neg-declared-unexhibited | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq
eff2-neg-formal-row-through-instance-2 | rewrite | retired syntax: deref, region
eff2-neg-installed-owner-read-omitted | rewrite | retired syntax: boxstore, deref, heap, lifetime, region, regionparam, replace, uniq; re-cite retired tags: SET-2
eff2-neg-undeclared-exhibited | rewrite | retired syntax: heap, lifetime, region, regionparam, uniq, vector
eff2-pos-borrowed-length-read-effect | rewrite | retired syntax: buffer, deref, measure_fn
eff2-pos-formal-row-through-instance-1 | rewrite | retired syntax: deref, region
eff2-pos-forwarded-storage-effect | rewrite | retired syntax: buffer, deref, measure_fn, uniq; re-cite retired tags: OWN-12
ent1-neg-instantiation-judged-at-value | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
ent1-pos-instantiation-judged-at-value | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
ent2-neg-counted-subscript-endpoint | rewrite | retired syntax: array_old
ent2-neg-expired-spelling-inherits-nothing | rewrite | retired syntax: measure_fn, owneffect, slice
ent2-neg-no-fact-across-call | rewrite | retired syntax: measure_fn, owneffect, slice
ent2-pos-a-run-capacity-survives-a-root-replace | rewrite | retired syntax: measure_fn, replace, uniq, vector; re-cite retired tags: SET-2
ent2-pos-array-length-survives-root-replace | rewrite | retired syntax: array_old, replace; re-cite retired tags: SET-2
ent2-pos-pool-static-and-boxed-capacity | rewrite | retired syntax: arena, boxstore, deref, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: BLK-3
ent3-neg-affine-requirement-disjunct | rewrite | declares `room`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
ent3-neg-affine-requirement-measure-replaced | rewrite | retired syntax: buffer, measure_fn, owneffect, replace
ent3-neg-affine-requirement-scalar-replaced | rewrite | declares `room`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
ent3-pos-band-check-decomposition | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
ent3-pos-bor-guard-decomposition | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
ent3-pos-s1-branch-fact | rewrite | retired syntax: measure_fn, owneffect, slice
ent3-pos-s11-counted-range-run | rewrite | retired syntax: vector
ent3-pos-s12-a-published-relation-reaches-a-let-binder | rewrite | declares `cap`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
ent3-pos-s12-a-published-relation-reaches-a-set-target | rewrite | declares `cap`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
ent3-pos-s4-requires-fact | rewrite | retired syntax: measure_fn, owneffect, slice
ent3-pos-s5-set-commit-image | rewrite | retired syntax: measure_fn, owneffect, slice
ent3-pos-s6-allocation-length-fact | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq, vector
ent4-neg-nonstrict-bound-underivable | rewrite | retired syntax: measure_fn, owneffect, slice
ent4-pos-contradictory-state-discharges | rewrite | retired syntax: measure_fn, owneffect, slice
ent4-pos-disequality-strengthens | rewrite | retired syntax: measure_fn, owneffect, slice
ent4-pos-transitivity-discharges | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-a-callee-write-through-a-uniq-extent-kills-the-room | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
ent5-neg-borrowed-field-reread-after-write | rewrite | retired syntax: deref, uniq
ent5-neg-callee-uniq-buffer-replace-kills-length | rewrite | retired syntax: buffer, deref, region, replace, uniq
ent5-neg-element-reread-after-write | rewrite | retired syntax: deref, measure_fn, slice, uniq
ent5-neg-else-free-guard-fallthrough | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-field-index-replacement | rewrite | retired syntax: measure_fn, owneffect, vector
ent5-neg-field-overflow-replacement | rewrite | retired syntax: owneffect
ent5-neg-join-missing-predecessor-bound | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-join-takes-weakest-bound | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-kill-on-write | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-loop-rule-drops-preloop-fact | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-nested-weaker-bound | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-value-if-nonstrict-index-delivery | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-neg-wrapped-index-replacement | rewrite | retired syntax: array_old, owneffect
ent5-pos-callee-sibling-fact | rewrite | retired syntax: deref, measure_fn, owneffect, region, uniq, vector
ent5-pos-close-before-scope-kill | rewrite | retired syntax: array_old, owneffect
ent5-pos-element-write-preserves-length | rewrite | retired syntax: region, uniq, vector
ent5-pos-else-free-return-bound | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-pos-field-write-survivors | rewrite | retired syntax: deref, measure_fn, owneffect, region, uniq, vector
ent5-pos-indexed-sibling-write | rewrite | retired syntax: deref, region, uniq, vector
ent5-pos-indexed-write-kill-1 | rewrite | retired syntax: deref, region, uniq, vector
ent5-pos-join-keeps-common-bound | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-pos-join-measure-identities | rewrite | retired syntax: deref, measure_fn, owneffect, slice, uniq
ent5-pos-nested-statement-joins | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-pos-partition-closure-before-join | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-pos-recheck-replacement | rewrite | retired syntax: deref, measure_fn, owneffect, slice, uniq, vector
ent5-pos-replacement-length-after-join | rewrite | retired syntax: region, replace, uniq, vector; re-cite retired tags: SET-2
ent5-pos-replacement-length-unrelated-binding-1 | rewrite | retired syntax: region, replace, uniq, vector; re-cite retired tags: SET-2
ent5-pos-replacement-length-unrelated-binding-2 | rewrite | retired syntax: region, replace, uniq, vector; re-cite retired tags: SET-2
ent5-pos-return-does-not-kill-loop-head-fact | rewrite | retired syntax: measure_fn, owneffect, slice
ent5-pos-returned-scalar-run-measure-1 | rewrite | retired syntax: borrowresult, deref, reflifetime, region, regionparam, uniq, vector; re-cite retired tags: OWN-14
ent5-pos-value-if-index-delivery | rewrite | retired syntax: measure_fn, owneffect, slice
ent6-neg-join-one-arm-advances-accumulator | rewrite | retired syntax: vector
ent6-pos-common-byte-image-constant-delta | rewrite | retired syntax: measure_fn, owneffect, slice
ent6-pos-join-value-if-lifted-addend | rewrite | retired syntax: vector
err2-neg-directory-result-arm | rewrite | retired syntax: deref, region, slice, uniq
err3-pos-continuation-run | rewrite | declares `next`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
ex1-pos-worked-example | rewrite | retired syntax: deref, region
exclusive-generic-whole-replacement | rewrite | retired syntax: deref, measure_fn, region, replace, uniq, vector; re-cite retired tags: BLK-4, SET-2
exclusive-neg-exit-is-not-own-increment | rewrite | retired syntax: deref, measure_fn, region, uniq, vector
exclusive-neg-generic-element-replacement-kills | rewrite | retired syntax: deref, measure_fn, region, replace, uniq, vector; re-cite retired tags: BLK-4
exclusive-neg-no-ensures-no-length-fact | rewrite | retired syntax: deref, measure_fn, region, uniq, vector
exclusive-neg-view-live-during-call | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
exclusive-neg-whole-referent-replace-kills | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
fn1-neg-borrowed-slice-result | rewrite | retired syntax: borrowresult, lifetime, reflifetime, regionparam, slice; re-cite retired tags: OWN-4
fn1-neg-contract-borrowed-slice-result | rewrite | retired syntax: borrowresult, lifetime, regionparam, slice, uniq
fn1-neg-result-provenance-other-kind | rewrite | retired syntax: borrowresult, deref, reflifetime, regionparam, uniq
fn1-neg-result-provenance-two-same-kind | rewrite | retired syntax: borrowresult, deref, regionparam, uniq
fn1-neg-result-provenance-two-same-kind-called | rewrite | retired syntax: borrowresult, deref, region, regionparam, uniq
fn1-neg-returned-slice-arena-origin | rewrite | retired syntax: arena, array_old, deref, lifetime, reflifetime, regionparam, slice; re-cite retired tags: OWN-10, OWN-5, STOR-4
fn1-neg-unused-formal-formation-2 | rewrite | retired syntax: borrowresult, reflifetime, regionparam
fn1-pos-distinct-provenance-regions | rewrite | retired syntax: borrowresult, deref, region, regionparam, uniq; re-cite retired tags: OWN-14
fn1-pos-exclusive-owner-exit-state | rewrite | retired syntax: allocates, boxstore, deref, heap, lifetime, region, regionparam, replace, uniq; re-cite retired tags: SET-2
fn1-pos-result-provenance-distinct-regions | rewrite | retired syntax: borrowresult, deref, region, regionparam, uniq; re-cite retired tags: OWN-6
fn1-pos-result-provenance-zero-candidate | rewrite | retired syntax: borrowresult, reflifetime, regionparam
fn1-pos-returned-slice-const-run | rewrite | retired syntax: lifetime, measure_fn, reflifetime, region, regionparam, slice, store_arg, vector; re-cite retired tags: OWN-10, OWN-5
fn1-pos-returned-slice-inputs-run | rewrite | retired syntax: lifetime, measure_fn, region, regionparam, slice, uniq, vector; re-cite retired tags: OWN-5
fn2-neg-function-argument-kind-5 | rewrite | retired syntax: boxstore, lifetime, regionparam
fn2-neg-function-region-bearing-targ | rewrite | retired syntax: lifetime, regionparam, slice
fn2-neg-nominal-region-bearing-targ | rewrite | retired syntax: slice
fn2-neg-two-stores-give-two-result-types | rewrite | retired syntax: allocates, arena, deref, lifetime, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8, PROV-1
fn2-pos-a-const-parameter-beside-a-region-parameter | rewrite | retired syntax: allocates, buffer, deref, fits, heap, lifetime, measure_fn, region, regionparam, uniq, vector; re-cite retired tags: BLK-1, FORM-8
fn2-pos-a-result-only-region-is-written-at-the-call | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8, PROV-1
fn2-pos-generic-store-confinement | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: OWN-10
fn2-pos-zero-run-element-brand | rewrite | retired syntax: arena, lifetime, region, store_arg, vector; re-cite retired tags: FORM-8
fn3-pos-normalized-region-effects | rewrite | retired syntax: measure_fn, owneffect, slice
fn4-neg-formal-region-position | rewrite | retired syntax: deref, measure_fn, vector
fn4-neg-formal-row-coverage | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq, vector
fn4-neg-input-owner-for-fresh-result | rewrite | retired syntax: vector
fn4-neg-member-effect-ordinals-2 | rewrite | retired syntax: deref, uniq
fn4-neg-member-region-bound | rewrite | retired syntax: deref, lifetime, measure_fn, regionparam, vector
fn4-neg-member-region-bound-3 | rewrite | retired syntax: deref, lifetime, measure_fn, regionparam, vector
fn4-neg-pure-member-binds-release | rewrite | retired syntax: deref, region, uniq
fn4-pos-formal-owned-result-transfer-2 | rewrite | retired syntax: vector
fn4-pos-member-effect-ordinals-1 | rewrite | retired syntax: deref, uniq
fn4-pos-member-region-bound-2 | rewrite | retired syntax: deref, lifetime, measure_fn, regionparam, vector
fn6-pos-a-generic-cycle-at-the-callers-own-parameters | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
fn7-pos-the-entry-receives-the-general-store | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq, vector; re-cite retired tags: BLK-2, PROV-1
fn8-neg-counted-call-past-end | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
fn8-neg-external-actual-without-fact | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
fn8-neg-length-requirement-distinct-objects | rewrite | retired syntax: measure_fn, owneffect, region, slice, uniq, vector
fn8-neg-requires-noncopy-local | rewrite | retired syntax: vector
fn8-neg-view-write-measure-transport-2 | rewrite | retired syntax: buffer, deref, measure_fn, region, uniq
fn8-neg-view-write-measure-transport-3 | rewrite | retired syntax: buffer, deref, measure_fn, region, uniq
fn8-neg-write-past-source | rewrite | retired syntax: buffer, region, slice, uniq
fn8-neg-write-range-unproved | rewrite | retired syntax: deref, region, slice, uniq
fn8-pos-affine-requirement-images | rewrite | declares `room`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
fn8-pos-affine-requirement-measures | rewrite | retired syntax: buffer, measure_fn, owneffect, region, slice, uniq
fn8-pos-affine-requirement-signed-leaves | rewrite | declares `room`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
fn8-pos-external-actual-after-branch | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
fn8-pos-loop-invariant-call-chain | rewrite | declares `last`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
fn8-pos-requires-affine-row | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: BLK-4, VIEW-1
fn8-pos-returned-referent-guard | rewrite | retired syntax: array_old, borrowresult, deref, reflifetime, regionparam; re-cite retired tags: OWN-14
fn8-pos-two-independent-input-lengths | rewrite | retired syntax: lifetime, measure_fn, region, regionparam, slice, uniq, vector
fn9-neg-aggregate-field-result-selector | rewrite | retired syntax: lifetime, measure_fn, regionparam, vector
fn9-neg-take-does-not-preserve-length | rewrite | retired syntax: deref, lifetime, measure_fn, region, regionparam, uniq, vector; re-cite retired tags: BLK-3
fn9-pos-a-measure-reader-in-return-position-over-a-run | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector; re-cite retired tags: BLK-1
fn9-pos-defined-run-last-index | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
fn9-pos-directory-result-relations | rewrite | retired syntax: deref, measure_fn, region, slice, uniq
fn9-pos-owned-run-return-through-loop | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
form3-pos-index-ordinary-ident | rewrite | retired syntax: region, uniq, vector
form3-pos-lexical-classes | rewrite | retired syntax: deref, region
form8-pos-a-construct-elides-the-region-its-field-determines | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8, PROV-1
form8-pos-a-construct-writes-the-region-no-field-determines | rewrite | retired syntax: lifetime, region, regionparam; re-cite retired tags: FORM-8
form8-pos-a-run-element-region-is-determined-by-the-actual | rewrite | retired syntax: arena, deref, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: BLK-1, FORM-8
form8-pos-all-result-region-ordinals | rewrite | retired syntax: boxstore, lifetime, regionparam; re-cite retired tags: FORM-8
form8-pos-captured-brand-before-loan | rewrite | retired syntax: boxstore, lifetime, regionparam; re-cite retired tags: FORM-8
form8-pos-elided-regions-run | rewrite | retired syntax: deref, region; re-cite retired tags: FORM-8, OWN-3, OWN-4
form8-pos-flat-element-result-brands | rewrite | retired syntax: boxstore, deref, lifetime, measure_fn, region, regionparam, replace, store_arg, uniq, vector; re-cite retired tags: FORM-8, SET-2
form8-pos-multiple-brand-positions | rewrite | retired syntax: deref, lifetime, region, regionparam; re-cite retired tags: FORM-8
form8-pos-nested-container-brands | rewrite | retired syntax: boxstore, deref, lifetime, measure_fn, regionparam, vector; re-cite retired tags: FORM-8
form8-pos-phantom-nested-brands | rewrite | retired syntax: deref, lifetime, region, regionparam; re-cite retired tags: FORM-8
form8-pos-related-pair-written | rewrite | retired syntax: borrowresult, deref, reflifetime, region, regionparam; re-cite retired tags: FORM-8, OWN-4
form8-pos-single-input-invariant-brand | rewrite | retired syntax: deref, lifetime, region, regionparam; re-cite retired tags: FORM-8, OWN-4
form8-pos-two-brand-nested-results | rewrite | retired syntax: boxstore, lifetime, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8
gram3-pos-modes | rewrite | retired syntax: deref, reflifetime, regionparam, uniq
gram4-pos-stmts | rewrite | retired syntax: region
gram5-pos-exprs-places | rewrite | retired syntax: deref, region
gram5-pos-recursive-place-projection | rewrite | retired syntax: deref, region
inv1-neg-a-measure-invariant-is-unproved | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
inv1-neg-a-subscript-inside-an-invariant-owes-its-bound | rewrite | retired syntax: measure_fn, vector
inv1-neg-an-affine-factor-is-not-a-measure-former | rewrite | retired syntax: measure_fn, uniq, vector
inv1-neg-static-capacity-is-not-initialized-length | rewrite | retired syntax: deref, measure_fn, region, uniq, vector
inv1-neg-unpublished-element-capacity | rewrite | retired syntax: arena, deref, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector
inv1-pos-a-measure-former-is-an-affine-factor | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
inv1-pos-a-subscript-inside-an-invariant-s-measure-place | rewrite | retired syntax: measure_fn, region, uniq, vector
inv1-pos-bounded-byte-sum | rewrite | retired syntax: measure_fn, owneffect, slice
inv1-pos-conditional-compaction-index | rewrite | retired syntax: deref, measure_fn, slice, uniq
inv1-pos-counted-ring-reset | rewrite | retired syntax: region, uniq, vector
inv1-pos-guarded-cursor-patterns | rewrite | retired syntax: measure_fn, owneffect, slice, vector
inv1-pos-header-measure-factor | rewrite | retired syntax: buffer, measure_fn, owneffect
inv1-pos-named-const-atom | rewrite | declares `cap`, `last`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
liv1-neg-branches-disagree-on-liveness | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
liv1-neg-loop-leaves-an-outer-binding-dead | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
liv1-pos-loop-moves-and-restores-an-outer-binding | rewrite | retired syntax: measure_fn, owneffect, uniq, vector; re-cite retired tags: LIV-2
liv2-neg-a-projected-set-target-declares-nothing | rewrite | re-cite retired tags: LIV-2
liv2-neg-one-element-target-is-read-out-at-most-once | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-a-set-target-that-declares-its-binding | rewrite | re-cite retired tags: LIV-2
liv2-pos-an-affine-element-read-out-swaps-two-slots | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-1, BLK-3, LIV-2
liv2-pos-box-referent-readout | rewrite | retired syntax: boxstore, deref, heap, lifetime, measure_fn, region, regionparam, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-entry-dead-owner-initialization | rewrite | retired syntax: owneffect; re-cite retired tags: LIV-2
liv2-pos-proved-dynamic-indices-swap-affine-elements | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-read-out-at-a-binding-a-field-and-a-deref | rewrite | retired syntax: allocates, boxstore, deref, heap, lifetime, measure_fn, region, regionparam, uniq, vector; re-cite retired tags: LIV-2, OWN-5
liv2-pos-read-out-keeps-the-root-and-its-other-fields | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-subscript-targets-commit-their-own-ordinals | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-swap-and-rotation | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: LIV-2
liv2-pos-two-elements-of-two-inner-runs-are-distinct-targets | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-1, LIV-2
liv2-pos-two-fields-of-one-root | rewrite | re-cite retired tags: LIV-2
msr1-neg-a-subscript-inside-a-measure-place-owes-its-obligation | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector
msr1-pos-a-measure-over-a-subscripted-place-is-a-term | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-1
msr1-pos-subscript-obligation-against-len | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
msr1-pos-the-four-measure-readers | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
msr2-neg-an-element-store-kills-the-element-s-own-length | rewrite | retired syntax: arena, lifetime, measure_fn, region, replace, store_arg, uniq, vector; re-cite retired tags: SET-2
msr2-neg-descriptor-write-kills-the-measure | rewrite | retired syntax: measure_fn, region, replace, uniq, vector; re-cite retired tags: SET-2
msr2-pos-an-element-write-kills-no-measure-of-the-run | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector; re-cite retired tags: BLK-1
msr2-pos-element-write-keeps-the-measure | rewrite | retired syntax: measure_fn, uniq, vector
msr2-pos-remeasure-after-descriptor-replace | rewrite | retired syntax: measure_fn, region, replace, uniq, vector; re-cite retired tags: SET-2
msr2-pos-sibling-field-write-keeps-the-measure | rewrite | retired syntax: measure_fn, uniq, vector
msr3-neg-a-boundary-row-carries-no-measure-through-the-slot | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
msr3-neg-a-rebind-carries-the-run-and-not-its-elements | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
msr3-neg-a-two-payload-enum-has-no-payload-place | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
msr3-neg-one-element-position-is-not-another | rewrite | the undischarged-requirement subject survives at ordinary PRE-1 constructor/window-operation calls: re-cite FN-8 (and MSR-4 for the goal), on Slots/Ring/Box vocabulary
msr3-pos-a-construct-carries-a-measure-two-levels-down | rewrite | retired syntax: measure_fn, region, uniq, vector
msr3-pos-a-construct-field-and-its-destructuring-carry-the-measures | rewrite | retired syntax: arena, lifetime, region, regionparam, store_arg, uniq, vector
msr3-pos-a-parameter-written-back-keeps-its-entry-measure | rewrite | retired syntax: measure_fn, owneffect, region, replace, uniq, vector; re-cite retired tags: SET-2
msr3-pos-a-payload-binder-carries-the-payload-s-measures | rewrite | retired syntax: arena, lifetime, region, store_arg, uniq, vector
msr3-pos-a-set-target-carries-the-value-s-measures | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: LIV-2
msr3-pos-an-element-position-keeps-the-value-put-into-it | rewrite | retired syntax: arena, lifetime, region, replace, store_arg, uniq, vector; re-cite retired tags: SET-2
msr3-pos-own-operand-call-datum | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
msr3-pos-uniq-state-measure-in-ensures | rewrite | retired syntax: arena, deref, lifetime, measure_fn, region, regionparam, store_arg, uniq
msr4-pos-capacity-requirement-discharges-a-length-obligation | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
msr5-pos-a-clause-side-is-an-affine-expression | rewrite | declares `next`, now reserved from every declaration role [FORM-3, TYPE-10]; rename the binder
msr5-pos-a-requirement-side-is-an-affine-expression | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
msr5-pos-measures-through-owned-transfers | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, replace, store_arg, uniq, vector; re-cite retired tags: OWN-5, SET-2
msr5-pos-two-measure-clause | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
msr6-neg-a-const-generic-affine-atom-is-unproved | rewrite | retired syntax: measure_fn, uniq, vector
msr6-pos-a-const-generic-is-an-affine-atom | rewrite | retired syntax: measure_fn, uniq, vector
msr6-pos-const-generic-in-a-clause-and-an-endpoint | rewrite | retired syntax: measure_fn, owneffect, uniq, vector
op1-neg-function-argument-kind-6 | rewrite | retired syntax: arena
op2-neg-unbounded-length-byte-sum | rewrite | retired syntax: measure_fn, owneffect, slice
op2-neg-zero-quotient-predecessor | rewrite | retired syntax: measure_fn, owneffect, vector
op4-neg-a-run-element-read-carries-its-obligation | rewrite | retired syntax: vector; re-cite retired tags: BLK-1
op4-neg-accumulator-subscript | rewrite | retired syntax: vector
op4-neg-array-view-domain-3 | rewrite | retired syntax: array_old, region, slice, uniq; re-cite retired tags: OWN-5, VIEW-1
op4-neg-byte-type-range-endpoint | rewrite | retired syntax: array_old, measure_fn, owneffect, slice
op4-neg-callee-minimum-subscript | rewrite | retired syntax: vector
op4-neg-ceiling-midpoint-endpoint | rewrite | retired syntax: measure_fn, owneffect, vector
op4-neg-constant-nested-field-bound-2 | rewrite | retired syntax: array_old
op4-neg-counted-accumulator-endpoint | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-counted-past-array-end | rewrite | retired syntax: array_old, owneffect
op4-neg-element-loan-boundary-1 | rewrite | retired syntax: borrowresult, reflifetime, region, regionparam, uniq, vector; re-cite retired tags: OWN-5
op4-neg-external-index-without-fact | rewrite | retired syntax: region, uniq, vector
op4-neg-index-invariant-before-replacement | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
op4-neg-index-undischarged | rewrite | retired syntax: owneffect, slice
op4-neg-indexed-write-kill-2 | rewrite | retired syntax: deref, region, uniq, vector
op4-neg-interposed-subscript | rewrite | retired syntax: array_old, deref, reflifetime, region, regionparam
op4-neg-join-wrapped-increment-bound | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-joined-out-of-range-constant | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-joined-zero-on-empty-view | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-nested-affine-past-end | rewrite | retired syntax: array_old, owneffect
op4-neg-nonstrict-descriptor-bound | rewrite | retired syntax: measure_fn, owneffect, vector
op4-neg-nonstrict-loop-head-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-ordinary-loop-exit-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-post-loop-cursor-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-post-loop-selected-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-neg-quotient-index-loose-bound | rewrite | retired syntax: measure_fn, owneffect, vector
op4-neg-remainder-other-descriptor | rewrite | retired syntax: measure_fn, owneffect, vector
op4-neg-returned-scalar-run-measure-2 | rewrite | retired syntax: borrowresult, deref, reflifetime, region, regionparam, replace, uniq, vector; re-cite retired tags: OWN-14
op4-neg-reverse-counted-past-end | rewrite | retired syntax: array_old, owneffect
op4-neg-unproved-data-dependent-scatter | rewrite | retired syntax: measure_fn, owneffect, slice
op4-pos-a-run-element-read-is-discharged-by-a-published-length | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-1
op4-pos-byte-type-range-index | rewrite | retired syntax: array_old, measure_fn, owneffect, slice
op4-pos-counted-window-bounds | rewrite | retired syntax: deref, measure_fn, owneffect, slice, uniq, vector
op4-pos-descriptor-index-images | rewrite | retired syntax: measure_fn, owneffect, vector
op4-pos-external-index-after-branch | rewrite | retired syntax: measure_fn, region, uniq, vector
op4-pos-index-discharged | rewrite | retired syntax: vector
op4-pos-midpoint-automatic | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
op4-pos-mirrored-counted-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-pos-nested-affine-index | rewrite | retired syntax: measure_fn, owneffect, slice
op4-pos-recursive-window-contract | rewrite | retired syntax: measure_fn, owneffect, vector
op4-pos-unreplaced-run-bound | rewrite | retired syntax: measure_fn, uniq, vector
op8-pos-integer-width-edges-run | rewrite | retired syntax: region
op9-neg-kernel-acquisition-without-a-fit-proof | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq, vector; re-cite retired tags: BLK-0, BLK-2
op9-pos-buffer-new | rewrite | retired syntax: allocates, buffer, deref, fits, heap, lifetime, region, regionparam, uniq, vector
op9-pos-generic-fit-boundary-run | rewrite | retired syntax: buffer, fits, heap, lifetime, regionparam, store_arg, vector
op9-pos-kernel-acquisition-with-a-fit-proof | rewrite | retired syntax: allocates, buffer, deref, fits, heap, lifetime, region, regionparam, uniq, vector; re-cite retired tags: BLK-0, BLK-2
own1-neg-an-unproved-candidate-index-pair-does-not-separate-a-cross-path | rewrite | retired syntax: array_old, deref, measure_fn, uniq, vector; re-cite retired tags: LIV-2
own1-neg-bare-affine-call | rewrite | retired syntax: region, uniq, vector
own1-neg-bare-uniq-copy | rewrite | retired syntax: uniq
own1-neg-index-atom-after-move | rewrite | retired syntax: vector
own1-neg-index-move-copy-offset | rewrite | retired syntax: region, uniq, vector
own1-neg-match-move-through-borrow | rewrite | moving out through a reference is still refused: matching through a reference leaves the scrutinee live and binds payloads as references [REF-1], so re-cite OWN-1/OWN-13
own1-neg-moved-directory-source | rewrite | retired syntax: deref, owneffect, region, slice, uniq
own1-neg-return-borrowed-box | rewrite | retired syntax: boxstore, deref, lifetime, regionparam
own1-neg-return-through-shared-borrow | rewrite | retired syntax: deref
own1-neg-set-rhs-kills-root | rewrite | retired syntax: owneffect
own1-pos-explicit-affine-call | rewrite | retired syntax: region, uniq, vector
own1-pos-match-projected-copy | rewrite | retired syntax: region, uniq, vector
own1-pos-moved-owner-local-origin | rewrite | retired syntax: deref, measure_fn, region, slice, uniq, vector; re-cite retired tags: VIEW-1
own1-pos-return-affine-contextual-move | rewrite | retired syntax: boxstore, lifetime, regionparam
own10-neg-dangle-caller | rewrite | the escaping-reference subject survives in a simpler form: drop the region machinery and re-cite REF-3 (a reference is never returned)
own10-neg-local-range-result-escape | rewrite | the escaping-reference subject survives in a simpler form: drop the region machinery and re-cite REF-3 (a reference is never returned)
own11-neg-borrow-outer-region | rewrite | retired syntax: reflifetime, region
own11-pos-loop-body-region | rewrite | retired syntax: deref; re-cite retired tags: FORM-8
own11-pos-loop-inner-region | rewrite | retired syntax: deref, region
own12-neg-a-view-argument-beside-a-unique-borrow-of-itself | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
own12-neg-alias-uniq-args | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
own12-pos-distinct-uniq-args | rewrite | two disjoint write paths at one call stay admitted: re-cite EFF-5
own13-pos-affine-payload-match-run | rewrite | retired syntax: deref, region
own13-pos-arm-scoped-payload-borrows-1 | rewrite | retired syntax: deref, region, uniq
own13-pos-arm-scoped-payload-borrows-2 | rewrite | retired syntax: deref, region
own13-pos-arm-scoped-payload-borrows-3 | rewrite | retired syntax: deref, region
own13-pos-borrow-affine-payload | rewrite | retired syntax: deref; re-cite retired tags: OWN-5
own13-pos-borrow-match-live | rewrite | retired syntax: deref
own13-pos-projected-buffer-match-run | rewrite | retired syntax: buffer, deref, measure_fn, region
own13-pos-uniq-match-payloads | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-5
own14-pos-discarded-result-run | rewrite | retired syntax: borrowresult, reflifetime, region, regionparam; re-cite retired tags: OWN-14
own2-pos-three-modes | rewrite | the surviving half (own and & modes, reading and writing through a reference) stays admitted: re-cite REF-1/SET-1 and drop &uniq and the region
own3-pos-two-nominals-name-one-region | rewrite | retired syntax: lifetime, regionparam, vector; re-cite retired tags: OWN-3, PROV-1
own4-neg-return-local-borrow | rewrite | the escaping-reference subject survives in a simpler form: drop the region machinery and re-cite REF-3 (a reference is never returned)
own5-neg-borrowed-inline-field-write-2 | rewrite | write authority is no longer a borrow marker: re-express as a write through a reference parameter whose row declares no writes of that path, re-cite SET-1/EFF-1
own5-neg-borrowed-inline-field-write-3 | rewrite | write authority is no longer a borrow marker: re-express as a write through a reference parameter whose row declares no writes of that path, re-cite SET-1/EFF-1
own5-neg-element-loan-boundary-2 | rewrite | the subject becomes use-after-invalidation: the write/move of the prefix is now legal and the later use of the reference is the rejection, re-cite REF-2
own5-neg-match-borrow-affine-payload-move | rewrite | moving out through a reference is still refused: matching through a reference leaves the scrutinee live and binds payloads as references [REF-1], so re-cite OWN-1/OWN-13
own5-pos-borrowed-inline-field-write-1 | rewrite | retired syntax: borrowresult, deref, reflifetime, region, regionparam, uniq, vector; re-cite retired tags: OWN-5
own5-pos-distinct-inline-elements | rewrite | retired syntax: borrowresult, deref, reflifetime, region, regionparam, uniq, vector; re-cite retired tags: OWN-5
own5-pos-nested-inline-field-loans | rewrite | retired syntax: deref, region, uniq, vector; re-cite retired tags: OWN-5
own5-pos-read-through-holder | rewrite | the surviving half (own and & modes, reading and writing through a reference) stays admitted: re-cite REF-1/SET-1 and drop &uniq and the region
own5-pos-recursive-adjacent-child-ranges | rewrite | retired syntax: buffer, measure_fn, owneffect, region, slice, uniq; re-cite retired tags: OWN-12, OWN-5, OWN-6, VIEW-2
own5-pos-rhs-borrow-is-disjoint-from-captured-target | rewrite | retired syntax: deref, region, uniq, vector; re-cite retired tags: OWN-5, OWN-6
own5-pos-statement-child-parent-1 | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-5, OWN-6
own5-pos-statement-child-parent-3 | rewrite | retired syntax: buffer, deref, measure_fn, region, uniq; re-cite retired tags: OWN-5, OWN-6
own6-pos-a-helper-re-lends-its-view-destination | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5, OWN-6, VIEW-1
own6-pos-owned-control-headers-return-temporaries | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-4, OWN-5, OWN-6
own6-pos-owned-match-header-loans-1 | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-6
own6-pos-owned-match-header-loans-2 | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-6
own6-pos-scalar-write-run | rewrite | retired syntax: deref, region, uniq; re-cite retired tags: OWN-6
own6-pos-statement-children-use-a-longer-local-region | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, replace, uniq; re-cite retired tags: OWN-3, OWN-4, OWN-6, SET-2
own7-pos-distinct-noverlap | rewrite | retired syntax: deref, region, uniq
par1-neg-two-exclusive-views-of-one-origin-in-one-window | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
par1-pos-a-view-argument-is-a-footprint-on-its-origin | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: BLK-2, VIEW-1, VIEW-2
par2-pos-runtime-stride-range-helper | rewrite | retired syntax: buffer, measure_fn, owneffect, region, slice, uniq; re-cite retired tags: OWN-5, VIEW-2
par3-pos-a-per-iteration-run-from-the-store-is-iteration-own | rewrite | retired syntax: arena, deref, measure_fn, region, slice, store_arg, uniq, vector; re-cite retired tags: BLK-2, OWN-5
pending-op9-buffer-new | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq, vector
pre1-pos-linear-opaque-ordinary-transfer | rewrite | retired syntax: deref, region, replace, uniq; re-cite retired tags: SET-2
prf1-neg-wrapped-index-certificate | rewrite | retired syntax: measure_fn, owneffect, vector
prf1-pos-integer-tightening-midpoint | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
prf1-pos-one-proof-many-consumers | rewrite | retired syntax: measure_fn, owneffect, vector
prov1-neg-a-provider-in-a-stored-position | rewrite | retired syntax: heap, regionparam; re-cite retired tags: PROV-1
prov1-neg-a-source-nominal-collides-with-a-container | rewrite | retired syntax: vector; re-cite retired tags: PROV-1
prov1-neg-two-runs-of-two-stores-at-one-formal-region | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8, PROV-1
prov1-pos-a-nominal-declares-a-store-region | rewrite | retired syntax: lifetime, regionparam, vector; re-cite retired tags: PROV-1
prov1-pos-a-store-branded-run-in-a-field | rewrite | retired syntax: heap, lifetime, regionparam, vector; re-cite retired tags: PROV-1
prov3-neg-an-append-while-a-copy-view-is-still-used | rewrite | the subject becomes use-after-invalidation: the write/move of the prefix is now legal and the later use of the reference is the rejection, re-cite REF-2
prov6-neg-a-dispose-with-no-provider-in-scope | rewrite | retired syntax: dispose, lifetime, regionparam, vector
prov6-neg-a-generic-run-scope-release-needs-an-empty-proof | rewrite | retired syntax: heap, lifetime, regionparam, uniq, vector
prov6-neg-a-partial-consume-cannot-abandon-a-symbolically-linear-member | rewrite | retired syntax: buffer
prov6-neg-a-partial-consume-cannot-abandon-storage-without-its-provider | rewrite | retired syntax: buffer, lifetime, regionparam, vector; re-cite retired tags: PROV-1
prov6-neg-a-proved-empty-run-still-needs-its-backing-provider | rewrite | retired syntax: dispose, lifetime, measure_fn, regionparam, vector
prov6-neg-a-region-argument-that-names-no-store | rewrite | retired syntax: deref, reflifetime, region, regionparam; re-cite retired tags: FORM-8
prov6-neg-a-region-parameter-bounded-copy | rewrite | retired syntax: deref, reflifetime, regionparam
prov6-neg-a-store-backed-run-reaches-an-exit-without-the-capability | rewrite | retired syntax: lifetime, measure_fn, regionparam, vector; re-cite retired tags: PROV-1
prov6-neg-actual-captured-bound-1 | rewrite | retired syntax: boxstore, lifetime, regionparam, store_arg
prov6-neg-actual-captured-bound-3 | rewrite | retired syntax: boxstore, lifetime, regionparam, store_arg
prov6-neg-an-empty-run-release-needs-a-proof | rewrite | retired syntax: dispose, heap, lifetime, regionparam, uniq, vector
prov6-neg-dispose-of-a-modifier-linear-node | rewrite | retired syntax: dispose, region, uniq, vector
prov6-neg-dispose-of-a-view | rewrite | retired syntax: dispose, region, slice, uniq, vector
prov6-neg-dispose-through-a-shared-borrow | rewrite | retired syntax: deref, dispose, region, uniq, vector; re-cite retired tags: OWN-5
prov6-neg-dispose-without-a-capability-leaf | rewrite | retired syntax: dispose
prov6-neg-dispose-without-the-declared-write | rewrite | retired syntax: allocates, dispose, heap, lifetime, region, regionparam, uniq, vector
prov6-neg-full-array-missing-element-provider | rewrite | retired syntax: array_old, boxstore, lifetime, regionparam; re-cite retired tags: PROV-1
prov6-neg-linear-value-partially-consumed | rewrite | retired syntax: dispose, region, uniq, vector; re-cite retired tags: LIV-2
prov6-neg-zero-extent-full-array-keeps-linear-obligation | rewrite | retired syntax: array_old
prov6-pos-a-proved-empty-fixed-run-releases-at-scope-exit | rewrite | retired syntax: measure_fn, region, vector
prov6-pos-a-proved-empty-generic-run-disposes-early | rewrite | retired syntax: allocates, dispose, heap, lifetime, measure_fn, region, regionparam, uniq, vector
prov6-pos-a-proved-empty-generic-run-releases-at-scope-exit | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, uniq, vector
prov6-pos-a-region-argument-names-a-bump-extent | rewrite | retired syntax: allocates, arena, deref, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: BLK-2, FORM-8
prov6-pos-a-run-visits-its-window-before-its-backing | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: BLK-1
prov6-pos-a-scope-holding-the-provider-takes-the-derived-release | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, uniq, vector
prov6-pos-actual-captured-bound-2 | rewrite | retired syntax: boxstore, lifetime, regionparam, store_arg
prov6-pos-an-early-release-of-a-store-backed-run | rewrite | retired syntax: allocates, dispose, heap, lifetime, region, regionparam, uniq, vector; re-cite retired tags: BLK-2
prov6-pos-buffer-release-cycle | rewrite | retired syntax: buffer, fits
prov6-pos-commit-reinitialises-the-consumed-sub-place | rewrite | retired syntax: allocates, dispose, heap, lifetime, region, regionparam, uniq, vector; re-cite retired tags: LIV-2
prov6-pos-consume-and-drain-on-every-result-arm | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-3
prov6-pos-destructuring-consume-discharges-the-obligation | rewrite | retired syntax: allocates, dispose, heap, lifetime, region, regionparam, uniq, vector
prov6-pos-dispose-runs-the-release-walk-early | rewrite | retired syntax: allocates, dispose, heap, lifetime, measure_fn, region, regionparam, uniq, vector
prov6-pos-dispose-writes-the-operands-storage-origin | rewrite | retired syntax: allocates, dispose, heap, lifetime, region, regionparam, uniq, vector
reject-own10-dangle | rewrite | the escaping-reference subject survives in a simpler form: drop the region machinery and re-cite REF-3 (a reference is never returned)
reject-sys14-list-end-beyond-buffer | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
reject-syseff-conditional-release-narrow | rewrite | retired syntax: deref, region, uniq
reject-syseff-declared-unexhibited | rewrite | retired syntax: region
reject-sysfile-permit-used-twice | rewrite | retired syntax: deref, region, uniq
reject-syshost-copybytes-end-beyond-buffer | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
reject-syshost-copybytes-start-after-end | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
reject-syshost-copybytes-start-beyond-buffer | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-ex1-value-match | rewrite | retired syntax: deref, region
run-exclusive-generic-append | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-3, BLK-4
run-exclusive-nested-field | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-4
run-exclusive-no-ensures-reread | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-3, BLK-4
run-exclusive-owning-map-put | rewrite | retired syntax: allocates, boxstore, deref, heap, lifetime, measure_fn, region, regionparam, replace, store_arg, uniq, vector; re-cite retired tags: BLK-3, BLK-4, SET-2
run-exclusive-push-pop-counted | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-3, BLK-4
run-exclusive-take-multiple-results | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-3
run-generic-owning-map-behavior | rewrite | retired syntax: allocates, boxstore, buffer, deref, dispose, fits, heap, lifetime, measure_fn, region, regionparam, replace, store_arg, uniq, vector; re-cite retired tags: BLK-3
run-generic-priority-behavior | rewrite | retired syntax: deref, measure_fn, region, replace, uniq, vector; re-cite retired tags: BLK-3
run-sysarg-count-and-get | rewrite | retired syntax: region, uniq
run-sysdir-open-notfound | rewrite | retired syntax: deref, region, uniq
run-sysfile-close-returns-permit | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysfile-empty | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysfile-exact | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysfile-failed-open-returns-permit | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysfile-multichunk | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysfile-short | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syshost-copybytes-toosmall-unchanged | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syshost-copyutf8-invalid-unchanged | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syshost-copyutf8-toosmall-unchanged | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syshost-nontext-argv-bytes-roundtrip | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syshost-nontext-argv-utf8-invalid | rewrite | retired syntax: region, uniq
run-sysin-read-to-end | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-sysout-basic-write | rewrite | retired syntax: measure_fn, region, slice, uniq, vector
run-sysout-redirect-same-sink-order | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
run-syspath-absolute-rejected | rewrite | retired syntax: region, uniq
run-syspath-dotdot-preserved | rewrite | retired syntax: region, uniq
run-syspath-relative-basic | rewrite | retired syntax: region, uniq
s16-pos-result-list-reaches-both-let-binders | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
s16-pos-result-list-reaches-both-set-targets | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
s39-neg-a-cell-reaches-an-exit-without-the-capability | rewrite | retired syntax: boxstore, lifetime, regionparam
s39-pos-a-cell-at-a-bump-extent-reclaims-with-its-region | rewrite | retired syntax: arena, deref, region, store_arg, uniq; re-cite retired tags: BLK-0, BLK-2
s39-pos-a-cell-holds-one-value-at-the-general-store | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq; re-cite retired tags: BLK-0, PROV-1
s7-pos-unsigned-remainder-bound-run | rewrite | retired syntax: vector
set1-neg-a-run-element-target-needs-its-subscript-bound | rewrite | retired syntax: region, uniq, vector; re-cite retired tags: BLK-1
set1-neg-array-view-domain-2 | rewrite | retired syntax: array_old, region, slice; re-cite retired tags: OWN-5, VIEW-1
set1-pos-a-run-element-is-a-set-target | rewrite | retired syntax: measure_fn, region, uniq, vector; re-cite retired tags: BLK-1
set1-pos-index-is-captured-before-rhs-borrow | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: OWN-5, OWN-6
set1-pos-owned-cell-content-target-1 | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq; re-cite retired tags: BLK-4, SET-2
set1-pos-owned-cell-content-target-2 | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, replace, uniq, vector; re-cite retired tags: BLK-4, SET-2
set2-neg-rhs-type-mismatch | rewrite | retired syntax: replace; re-cite retired tags: SET-2
set2-pos-a-run-element-is-a-replace-target | rewrite | retired syntax: region, replace, uniq, vector; re-cite retired tags: BLK-1, SET-2
set2-pos-affine-field-replace | rewrite | retired syntax: replace; re-cite retired tags: SET-2
set2-pos-box-descriptor-replace | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, replace, uniq; re-cite retired tags: SET-2
set2-pos-replacement-root-live | rewrite | retired syntax: measure_fn, replace, uniq, vector; re-cite retired tags: SET-2
stor1-neg-an-affine-element-target-with-no-read-out | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: BLK-1, LIV-2
stor1-neg-whole-buffer-replacement | rewrite | retired syntax: buffer, deref, uniq
stor1-neg-whole-cell-replacement | rewrite | retired syntax: boxstore, deref, lifetime, regionparam, uniq
stor2-pos-arena-delivery-run | rewrite | retired syntax: arena, deref, lifetime, region, store_arg; re-cite retired tags: STOR-2
stor2-pos-box-bool-read-run | rewrite | retired syntax: deref; re-cite retired tags: STOR-2
stor2-pos-box-new | rewrite | `Box<T>` and `box_new` survive unbranded: drop the store argument and the Result wrapper, re-cite OP-13/TYPE-9/STOR-8
stor3-pos-box-drop-region | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq
stor3-pos-empty-vector-run | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, uniq, vector
stor5-neg-arena-new-region-bearing | rewrite | retired syntax: arena, region, slice, store_arg, uniq, vector; re-cite retired tags: STOR-2
stor5-neg-box-new-arena-content | rewrite | retired syntax: allocates, arena, heap, lifetime, region, regionparam, store_arg, uniq; re-cite retired tags: STOR-2
stor5-neg-box-new-region-bearing | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, slice, uniq; re-cite retired tags: STOR-2
stor5-neg-zero-extent-array-cannot-hide-provider | rewrite | retired syntax: array_old, heap, lifetime, regionparam
stor5-pos-direct-nested-slice-type | rewrite | retired syntax: measure_fn, slice
sys14-directory-release | rewrite | retired syntax: deref, region, uniq
sys14-entry-kind-closed | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
sys14-list-handle-affine | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
sys14-list-handle-unique | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
sys14-list-outcome-exhaustive | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
sys14-list-zero-range | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
sys14-no-path-from-bytes | rewrite | retired syntax: region, uniq, vector
sys14-open-directory-component | rewrite | retired syntax: deref, region, slice, uniq, vector
sys14-open-directory-empty-name | rewrite | retired syntax: deref, region, slice, uniq, vector
sys14-open-directory-success | rewrite | retired syntax: deref, region, slice, uniq, vector
sysin-read-outcome-exhaustive | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
sysin-read-zero-range | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
systcp-accept-permit-returned | rewrite | retired syntax: deref, region, uniq
systcp-connect-permit-returned | rewrite | retired syntax: deref, region, uniq
systcp-connection-field-effect-paths | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
systcp-connection-moved-half-kills-binding | rewrite | retired syntax: deref, region, uniq
systcp-connection-two-halves | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector; re-cite retired tags: OWN-5
systcp-listen-permit-returned | rewrite | retired syntax: deref, region, uniq
type2-neg-an-affine-element-moved-outside-its-own-commit | rewrite | retired syntax: arena, lifetime, measure_fn, region, store_arg, uniq, vector; re-cite retired tags: LIV-2
type2-pos-a-nominal-is-generic-over-its-store | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: FORM-8, PROV-1
type2-pos-affine-array-elements | rewrite | retired syntax: array_old
type2-pos-buffer-tagonly | rewrite | retired syntax: vector
type2-pos-generic-affine-array-member | rewrite | retired syntax: array_old
type2-pos-recursive-array-result-run | rewrite | retired syntax: array_old, owneffect
type3-pos-nameable | rewrite | retired syntax: reflifetime, regionparam, uniq
type5-neg-array-view-domain-6 | rewrite | retired syntax: array_old, region, slice; re-cite retired tags: OWN-5, VIEW-1
type5-neg-function-argument-kind-1 | rewrite | retired syntax: boxstore, lifetime, regionparam
type5-neg-function-argument-kind-2 | rewrite | retired syntax: boxstore, lifetime, regionparam
type5-neg-function-argument-kind-3 | rewrite | retired syntax: boxstore, lifetime, regionparam, vector
type5-neg-function-argument-kind-4 | rewrite | retired syntax: boxstore, lifetime, regionparam, vector
type5-neg-index-offset-type | rewrite | retired syntax: region, uniq, vector
type5-neg-repeated-brand-position-mismatch | rewrite | retired syntax: deref, lifetime, region, regionparam; re-cite retired tags: FORM-8
type5-neg-shared-for-uniq-arg | rewrite | retired syntax: region, uniq; re-cite retired tags: OWN-2
type5-neg-two-nominal-instances-at-two-regions | rewrite | retired syntax: arena, lifetime, measure_fn, region, regionparam, store_arg, uniq, vector; re-cite retired tags: PROV-1
type5-neg-vector-as-relative-path | rewrite | retired syntax: lifetime, regionparam, vector
type5-pos-index-element-type-derived | rewrite | retired syntax: array_old, owneffect
type5-pos-inferred-buffer-element | rewrite | retired syntax: buffer, measure_fn
type7-neg-a-run-holder-written-where-the-run-is-required | rewrite | retired syntax: deref, measure_fn, region, uniq, vector; re-cite retired tags: BLK-1
type7-neg-deref-nonref | rewrite | retired syntax: deref
type7-neg-implicit-read | rewrite | retired syntax: region
type7-neg-index-box-holder | rewrite | retired syntax: boxstore, lifetime, regionparam, vector
type7-neg-index-reference-holder | rewrite | retired syntax: region, vector
type7-neg-match-borrow-expression | rewrite | retired syntax: region
type7-neg-match-box-holder | rewrite | retired syntax: boxstore, lifetime, regionparam
type7-neg-match-reference-call | rewrite | retired syntax: borrowresult, reflifetime, regionparam
type7-neg-match-reference-holder | rewrite | retired syntax: region
type7-neg-propagate-box-holder | rewrite | retired syntax: boxstore, lifetime, regionparam
type7-neg-propagate-reference-holder | rewrite | still refused, but not by TYPE-7: v0.60 TYPE-7 owns only the Box-content step, so re-cite ERR-3/OWN-1 (propagate consumes, and a place rooted through a reference is not consumable)
type7-neg-return-box-as-referent | rewrite | retired syntax: boxstore, lifetime, regionparam
type7-pos-deref | rewrite | retired syntax: deref, region
v033-neg-allocation-fit-unproved | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq, vector
v033-pos-shared-contract-define | rewrite | retired syntax: measure_fn, region, uniq, vector
v033-pos-uninhabited-contract | rewrite | retired syntax: region, uniq, vector
v033-run-open-file-directory | rewrite | retired syntax: deref, region, slice, uniq, vector
v033-run-open-file-regular | rewrite | retired syntax: deref, region, slice, uniq, vector
v033-run-system-nonzero-next | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector
view1-neg-a-move-of-a-shared-view | rewrite | retired syntax: measure_fn, region, slice, uniq, vector; re-cite retired tags: VIEW-1
view1-neg-an-element-write-through-a-shared-view | rewrite | retired syntax: region, slice, uniq, vector; re-cite retired tags: VIEW-1
view1-pos-a-shared-view-is-used-twice-without-move | rewrite | retired syntax: measure_fn, owneffect, region, slice, uniq, vector; re-cite retired tags: OWN-5, VIEW-1
view1-pos-an-element-write-through-an-exclusive-view | rewrite | retired syntax: arena, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-5, VIEW-1, VIEW-2
view1-pos-array-view-domain-1 | rewrite | retired syntax: array_old, region, slice, uniq; re-cite retired tags: OWN-5, VIEW-1
view1-pos-captured-endpoints-run | rewrite | retired syntax: measure_fn, region, slice, vector; re-cite retired tags: VIEW-1
view2-neg-an-exclusive-view-of-a-named-const | rewrite | retired syntax: region, slice, uniq, vector; re-cite retired tags: OWN-5, VIEW-2
view2-neg-an-exclusive-view-takes-a-unique-borrow | rewrite | retired syntax: region, slice, uniq, vector; re-cite retired tags: VIEW-2
view2-neg-range-end-outside-source | rewrite | range formation keeps its two obligations `lo <= hi` and `hi <= x.len`: re-cite REF-4 and write `&x[lo..hi]`
view2-neg-reversed-relative-range | rewrite | range formation keeps its two obligations `lo <= hi` and `hi <= x.len`: re-cite REF-4 and write `&x[lo..hi]`
view2-pos-a-view-over-a-run | rewrite | retired syntax: measure_fn, region, slice, uniq, vector; re-cite retired tags: BLK-0, BLK-1, VIEW-2
view2-pos-an-exclusive-view-over-a-run | rewrite | retired syntax: region, slice, uniq, vector; re-cite retired tags: VIEW-1, VIEW-2
view2-pos-an-exclusive-view-over-an-array | rewrite | retired syntax: region, slice, uniq, vector; re-cite retired tags: VIEW-1, VIEW-2
view2-pos-captured-relative-and-empty-ranges | rewrite | retired syntax: buffer, measure_fn, region, slice; re-cite retired tags: VIEW-1, VIEW-2
view6-pos-a-helper-publishes-the-child-of-its-destination | rewrite | retired syntax: arena, deref, lifetime, measure_fn, reflifetime, region, regionparam, slice, store_arg, uniq, vector; re-cite retired tags: OWN-6, VIEW-2, VIEW-6
x-array-const-checksum-run | rewrite | retired syntax: vector
x-array-mutable-checksum-run | rewrite | retired syntax: measure_fn, uniq, vector
x-base64-rfc-vectors-run | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-10, OWN-12, OWN-3, OWN-4, OWN-5
x-borrow-return-uniq-local-region | rewrite | the escaping-reference subject survives in a simpler form: drop the region machinery and re-cite REF-3 (a reference is never returned)
x-borrow-two-shared-reads-run | rewrite | the surviving half (own and & modes, reading and writing through a reference) stays admitted: re-cite REF-1/SET-1 and drop &uniq and the region
x-borrow-uniq-shared-call-args-overlap | rewrite | the overlapping-argument subject survives as the pairwise call-site check: drop &uniq, declare the write in the row, re-cite EFF-5
x-borrow-write-through-shared-borrow | rewrite | write authority is no longer a borrow marker: re-express as a write through a reference parameter whose row declares no writes of that path, re-cite SET-1/EFF-1
x-borrowed-pool-tree-run | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-10, OWN-12, OWN-3, OWN-4, OWN-5
x-buffer-borrowed-columns-run | rewrite | retired syntax: allocates, deref, heap, lifetime, measure_fn, region, regionparam, slice, uniq, vector; re-cite retired tags: OWN-10, OWN-12, OWN-3, OWN-4, OWN-5
x-buffer-mutable-checksum-run | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, uniq, vector
x-child-reborrow-run | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-10, OWN-12, OWN-3, OWN-4, OWN-5, OWN-6
x-crc32-standard-vector-run | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
x-eff-allocating-fn-calls-only-pure | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq
x-eff-dup-reads-effect | rewrite | retired syntax: owneffect
x-eff-pure-combined-with-allocation | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq
x-eff-pure-fn-calls-allocating-fn | rewrite | retired syntax: allocates, deref, heap, lifetime, region, regionparam, uniq, vector
x-eff-trailing-comma-row | rewrite | retired syntax: allocates, heap, lifetime, region, regionparam, uniq
x-enum-borrow-payload-live | rewrite | retired syntax: deref, region
x-fn-own-arg-for-ref-param | rewrite | retired syntax: deref, region
x-fn-ref-arg-for-own-param | rewrite | retired syntax: region
x-integ-coin-borrow-match-score-twice | rewrite | retired syntax: deref, region
x-option-byte-scanner-run | rewrite | retired syntax: measure_fn, owneffect, region, slice, uniq, vector; re-cite retired tags: OWN-12
x-ownmove-borrow-after-match-move | rewrite | retired syntax: region
x-ownmove-borrow-match-live-then-own-move | rewrite | retired syntax: deref, region
x-requires-output-capacity-run | rewrite | retired syntax: arena, deref, measure_fn, owneffect, region, slice, store_arg, uniq, vector; re-cite retired tags: OWN-12
x-result-buffer-transform-run | rewrite | retired syntax: measure_fn, owneffect, region, uniq, vector
x-struct-cross-fn | rewrite | retired syntax: owneffect
x-struct-of-buffers-checksum-run | rewrite | retired syntax: allocates, heap, lifetime, measure_fn, region, regionparam, uniq, vector
x-typ-borrow-arg-missing-deref | rewrite | retired syntax: region
x-typ-uniq-deref-write-roundtrip | rewrite | retired syntax: deref, region, uniq
x-typ-value-where-borrow-param | rewrite | retired syntax: region
x-wc-chunk-summary-run | rewrite | retired syntax: deref, measure_fn, owneffect, reflifetime, region, regionparam, uniq, vector; re-cite retired tags: OWN-10, OWN-12, OWN-3, OWN-4, OWN-5
```

### retire (115)

```
blk0-neg-a-written-argument-the-row-does-not-declare | retire | the BLK-0 kernel declaration domain is deleted; prelude window operations are ordinary PRE-1 records whose rows are fixed, so `a written argument the row does not declare` has no kernel-row subject
blk1-neg-a-construct-names-a-provider | retire | providers/stores are deleted; there is no provider nominal to name in a construct
blk1-pos-a-store-backed-run-is-a-run-element | retire | every rule this case cites is retired in v0.60 (BLK-1,BLK-2,PROV-1); its subject is a deleted mechanism
blk2-neg-a-reservation-inside-a-loop-of-its-block | retire | frame-resident arena extents and their region placement are deleted with BLK-2
blk2-neg-a-reservation-names-a-region-parameter | retire | frame-resident arena extents and their region placement are deleted with BLK-2
blk2-neg-an-extent-reserved-at-a-caller-region | retire | frame-resident arena extents and their region placement are deleted with BLK-2
blk4-pos-a-unique-parameter-reaches-a-run-through-a-field | retire | every rule this case cites is retired in v0.60 (BLK-4); its subject is a deleted mechanism
blk4-pos-an-exclusive-provider-parameter-preserves-store-identity | retire | every rule this case cites is retired in v0.60 (BLK-2,BLK-4,PROV-1); its subject is a deleted mechanism
form3-neg-region-param-missing-apostrophe | retire | region parameters are deleted; the `['r]` spelling FORM-3 governed has no subject in v0.60
form8-neg-a-construct-writes-the-region-its-field-determines | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-elided-related-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-elided-result-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-region-block-is-the-loop-body | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-unreferenced-region-block-name | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-written-determined-region-argument | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-written-innermost-borrow-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-neg-written-unrelated-parameter-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
form8-pos-borrowed-array-view-brand-1 | retire | every rule this case cites is retired in v0.60 (FORM-8,VIEW-1); its subject is a deleted mechanism
form8-pos-borrowed-array-view-brand-2 | retire | every rule this case cites is retired in v0.60 (FORM-8,VIEW-1); its subject is a deleted mechanism
form8-pos-elided-store-brand | retire | every rule this case cites is retired in v0.60 (FORM-8); its subject is a deleted mechanism
form8-pos-indexed-argument-brands | retire | every rule this case cites is retired in v0.60 (FORM-8,OWN-12); its subject is a deleted mechanism
form8-pos-input-brand-inference | retire | every rule this case cites is retired in v0.60 (FORM-8); its subject is a deleted mechanism
form8-pos-longer-mode-loan-order-1 | retire | every rule this case cites is retired in v0.60 (FORM-8,OWN-12); its subject is a deleted mechanism
form8-pos-longer-mode-loan-order-2 | retire | every rule this case cites is retired in v0.60 (FORM-8,OWN-12); its subject is a deleted mechanism
form8-pos-narrower-loop-region-block | retire | every rule this case cites is retired in v0.60 (FORM-8,OWN-3); its subject is a deleted mechanism
inv1-neg-equality-relation | retire | INV-1 now admits `==` in a header_invariant and an invariant_stmt target (tag ledger section 4a); the refusal this case asserts is an acceptance in v0.60
liv2-neg-two-subscripted-targets-of-one-inner-run-overlap | retire | the multi-target `set` is deleted; LIV-2's two-target overlap judgment has no construct left to judge
liv2-neg-two-subscripts-of-one-run | retire | the multi-target `set` is deleted; LIV-2's two-target overlap judgment has no construct left to judge
own10-pos-local-region | retire | every rule this case cites is retired in v0.60 (OWN-10); its subject is a deleted mechanism
own12-pos-a-view-parameter-beside-a-unique-store | retire | every rule this case cites is retired in v0.60 (FORM-8,OWN-12,OWN-3); its subject is a deleted mechanism
own13-neg-borrowed-header-keeps-parent-suspended | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own3-neg-undeclared-signature-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
own3-neg-unknown-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
own3-pos-outlives-store | retire | every rule this case cites is retired in v0.60 (OWN-3); its subject is a deleted mechanism
own4-neg-brand-not-shortened-by-loan | retire | the region-outlives relation it violates is gone; forming a reference to a local is unconditionally admitted in v0.60 [REF-1]
own4-pos-outer-region-held-inside-1 | retire | every rule this case cites is retired in v0.60 (OWN-10,OWN-4); its subject is a deleted mechanism
own4-pos-outer-region-held-inside-2 | retire | every rule this case cites is retired in v0.60 (OWN-10,OWN-4); its subject is a deleted mechanism
own4-pos-return-caller-borrow | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-4); its subject is a deleted mechanism
own5-neg-a-published-child-freezes-its-parent-view | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-a-unique-borrow-of-a-parent-view-while-its-child-lives | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-array-view-domain-4 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-array-view-domain-5 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-captured-index-loan | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-captured-range-endpoint-reassignment | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-commit-overlaps-rhs-temporary | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-element-loan-region-1 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-empty-range-keeps-backing-owner | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-indexed-result-origin-2 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-later-argument-uses-suspended-parent | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-later-rhs-reuses-temporary-borrow | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-overlapping-child-ranges | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-owned-header-keeps-bound-borrow | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-owned-header-keeps-result-parent-suspended | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-owned-header-keeps-view-origin | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-parent-read-during-exclusive-child | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-read-while-uniq | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-replace-overlaps-rhs-temporary | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-replace-rhs-loan-commit | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-rhs-loan-duration-1 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-rhs-loan-duration-2 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-rhs-loan-duration-3 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-rhs-loan-duration-4 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-rhs-loan-duration-5 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-slice-value-match | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-statement-child-parent-2 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-statement-child-parent-4 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-statement-child-parent-5 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-statement-child-parent-6 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-statement-child-parent-7 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-view-descriptor-rhs-loan-1 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-view-descriptor-rhs-loan-2 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-neg-view-descriptor-rhs-loan-3 | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own5-pos-a-unique-borrow-of-a-parent-view-after-its-child | retire | every rule this case cites is retired in v0.60 (OWN-5,VIEW-1,VIEW-2); its subject is a deleted mechanism
own5-pos-borrowed-view-parent | retire | every rule this case cites is retired in v0.60 (OWN-5,VIEW-1); its subject is a deleted mechanism
own5-pos-element-loan-region-2 | retire | every rule this case cites is retired in v0.60 (OWN-4,OWN-5); its subject is a deleted mechanism
own5-pos-indexed-result-origin-1 | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-5); its subject is a deleted mechanism
own5-pos-inline-view-sibling | retire | every rule this case cites is retired in v0.60 (OWN-5,VIEW-1); its subject is a deleted mechanism
own5-pos-rhs-loan-duration-6 | retire | every rule this case cites is retired in v0.60 (LIV-2,OWN-5,OWN-6); its subject is a deleted mechanism
own5-pos-rhs-loan-duration-7 | retire | every rule this case cites is retired in v0.60 (LIV-2,OWN-5,OWN-6); its subject is a deleted mechanism
own5-pos-rhs-loan-duration-8 | retire | every rule this case cites is retired in v0.60 (LIV-2,OWN-5,OWN-6); its subject is a deleted mechanism
own5-pos-view-descriptor-rhs-loan-4 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6,VIEW-1); its subject is a deleted mechanism
own6-neg-exclusive-child-of-shared-range | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
own6-pos-callresult-borrow-chain | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-5,OWN-6); its subject is a deleted mechanism
own6-pos-callscoped-temp | retire | every rule this case cites is retired in v0.60 (OWN-6); its subject is a deleted mechanism
own6-pos-returned-borrow-chains-1 | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-6); its subject is a deleted mechanism
own6-pos-returned-borrow-chains-2 | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-6); its subject is a deleted mechanism
own6-pos-returned-borrow-chains-3 | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-6); its subject is a deleted mechanism
own6-pos-returned-borrow-chains-4 | retire | every rule this case cites is retired in v0.60 (OWN-14,OWN-6); its subject is a deleted mechanism
own6-pos-statement-child-resumption-1 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6); its subject is a deleted mechanism
own6-pos-statement-child-resumption-2 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6); its subject is a deleted mechanism
own6-pos-statement-child-resumption-3 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6); its subject is a deleted mechanism
own6-pos-statement-child-resumption-4 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6); its subject is a deleted mechanism
own6-pos-statement-child-resumption-5 | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6); its subject is a deleted mechanism
prov1-neg-a-second-store-in-one-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
prov1-neg-an-extent-elides-its-store-region | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
prov1-neg-no-implicit-entry-store-brand | retire | regions, region parameters and store brands do not exist in v0.60; FORM-8/OWN-3/PROV-1 are retired with no successor
prov3-pos-an-append-after-a-copy-view-went-dead | retire | every rule this case cites is retired in v0.60 (BLK-3,OWN-5,VIEW-1); its subject is a deleted mechanism
set2-neg-arena-replace-target | retire | the `replace` statement is deleted with SET-2; the exchange job moved to swap [OP-11] and `set p = f(p,..)` [OP-12], which reject on different premises
set2-neg-copy-target | retire | the `replace` statement is deleted with SET-2; the exchange job moved to swap [OP-11] and `set p = f(p,..)` [OP-12], which reject on different premises
set2-neg-region-bearing-target | retire | the `replace` statement is deleted with SET-2; the exchange job moved to swap [OP-11] and `set p = f(p,..)` [OP-12], which reject on different premises
set2-pos-unique-element-replacement | retire | every rule this case cites is retired in v0.60 (OWN-6,SET-2); its subject is a deleted mechanism
stor2-pos-arena-break-run | retire | every rule this case cites is retired in v0.60 (FORM-8,STOR-2); its subject is a deleted mechanism
stor2-pos-arena-control-run | retire | every rule this case cites is retired in v0.60 (FORM-8,STOR-2); its subject is a deleted mechanism
stor4-neg-arena-escape | retire | arenas and arena confinement are deleted (STOR-4 retired)
stor4-neg-borrowed-arena-result | retire | arenas and arena confinement are deleted (STOR-4 retired)
stor4-pos-arena-confined | retire | every rule this case cites is retired in v0.60 (STOR-4); its subject is a deleted mechanism
type7-neg-return-reference-holder | retire | TYPE-7 now reads a reference bare, so `return holder;` on a `&i32` parameter is admitted; the refusal this case asserts is an acceptance
view2-neg-an-element-write-while-a-child-view-lives | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
view2-neg-two-exclusive-views-of-one-place | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
view2-pos-a-shared-child-reborrow-of-an-exclusive-view | retire | every rule this case cites is retired in v0.60 (OWN-5,OWN-6,VIEW-1,VIEW-2); its subject is a deleted mechanism
view2-pos-two-shared-views-of-one-place | retire | every rule this case cites is retired in v0.60 (OWN-5,VIEW-1,VIEW-2); its subject is a deleted mechanism
view4-neg-a-set-at-a-view-binding | retire | view values are deleted: a reference is never a stored binding and never a result [TYPE-8, REF-3]
view6-neg-two-same-region-view-results | retire | view values are deleted: a reference is never a stored binding and never a result [TYPE-8, REF-3]
x-borrow-own-param-escape-no-return | retire | the region-outlives relation it violates is gone; forming a reference to a local is unconditionally admitted in v0.60 [REF-1]
x-borrow-two-uniq-same-place | retire | loan liveness, reference exclusivity, child reborrows and parent suspension are deleted: in v0.60 two references to one place coexist freely, and the only overlap judgment is the pairwise one at a call [EFF-5]
```
