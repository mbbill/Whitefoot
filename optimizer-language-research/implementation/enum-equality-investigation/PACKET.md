# Owner-decision packet — tag-only enum equality (proposed v0.8)

Status: **NOT APPROVED, NOT APPLIED**

Date: 2026-07-19

This packet assembles the evidence and gates for an owner decision. It does not
grant approval and does not authorize a specification, checker, compiler,
conformance, oracle, test-expectation, or roadmap change.

## 1. Decision requested

Choose one of these outcomes:

1. **Approve the exact v0.7 to v0.8 delta** in
   `V0.8-DELTA-DRAFT.md`, adding `eeq<T>` and `ene<T>` for exact
   same-type tag-only enums, including `Bool`; or
2. **Decline the spec delta**, retain v0.7, and authorize a separate choice
   between the externally guarded mapper rewrite and the fully structural
   6,952-pair-arm rewrite for the 21 non-`Bool` types measured in `DOSSIER.md`.

Approval of this packet's direction is not approval to edit the numbered
specification. A spec landing requires explicit owner approval of the exact
text in `V0.8-DELTA-DRAFT.md`, followed by the logged `make approve-spec`
workflow.

## 2. Recommendation

Recommend outcome 1: add the separately named `eeq`/`ene` family.

The new name preserves OP-7's domain discipline:

- `i*` remains integer;
- `f*` remains float;
- `b*` remains Bool logic; and
- `e*` denotes tag-only enum comparison, with `Bool` included because it is a
  tag-only enum.

Do not widen `ieq`/`ine`. Existing invalid wfc source is migration evidence,
not a reason to make the integer prefix mean two things.

## 3. Evidence summary

| Leg | Result | Source |
|---|---|---|
| Accepted-language audit | v0.7 OP-1 limits `ieq`/`ine` to integer `T`; OP-7 assigns `i` to the integer domain | `spec/kernel-spec-v0.7.md` |
| Current-source census | Investigation baseline: 253 non-integer `ieq` sites in 93 functions and 18 files, over 22 tag-only types including `Bool`; the intervening v0.7 slice leaves 251 sites in 92 functions and the same 18 files/22 types; re-census required before landing | `DOSSIER.md` §3 |
| v0.7 compact rewrite projection | 21 mappers, 262 exhaustive arms, 484 mapper calls, approximately 1,367 added canonical lines; duplicate result codes are not rejected by v0.7 and need a separate injectivity guard | `DOSSIER.md` §4 |
| v0.7 structural rewrite projection | 6,952 nested-match variant-pair arms across the 21 non-`Bool` types, with 11 `Bool` sites using existing Boolean operations; checker-visible and injective by construction, but substantially larger | `DOSSIER.md` §4 |
| Focused raw IR | direct three-variant comparison is one `icmp eq i32`; mapper caller has two helper calls, two stack temporaries, and `icmp eq i64`, plus the branching helper | `DOSSIER.md` §5 |
| Optimized assembly observation | clang `-O2` recovered one caller comparison but retained the helper definition; no timing or byte-count claim | `DOSSIER.md` §5 |
| Exact spec mass | in-memory v0.8 materialization is 16,670 `cl100k_base` tokens versus 16,260 for v0.7 (+410), with 91 rules unchanged | `V0.8-DELTA-DRAFT.md` §10 |
| Soundness | same nominal tag-only type, valid discriminants, pure total compare, no payload, no ordering, no conversion | `DOSSIER.md` §7 and §5 below |

## 4. Exact semantic contract

The proposed operation family has these complete boundaries:

1. The operations are `eeq<T>(left, right)` and `ene<T>(left, right)`.
2. `T` is explicit and resolves to one nominal tag-only enum: every variant of
   `T` is nullary.
3. `Bool` is admitted by the same rule, not by an exception.
4. Both operands have exactly type `T`; no subtype, conversion, common
   representation, or cross-enum comparison exists.
5. Both operations return `own Bool` and have effect `pure`.
6. `eeq` returns `True()` exactly when both values denote the same declared
   variant of `T`.
7. `ene` is the exact boolean complement of `eeq` for every valid pair.
8. Payload-carrying enums, structs, primitives, integers, floats, arrays,
   buffers, and borrows are outside the domain.
9. No enum ordering operation is added. Existing `ilt`/`ile`/`igt`/`ige`
   remain integer-only.
10. `ieq` and `ine` remain integer-only. The new family does not legalize any
    current non-integer use of those spellings.
11. FN-8 admits the new operations under its existing pure-total operation-row
    rule; no special `requires` semantics or fact kind is added.

## 5. Soundness and defined-lowering argument

### Source-level invariant

OWN-1 classifies a tag-only enum as Copy. TYPE-2 and the enum declaration fix
its finite set of nullary variants. Accepted construction and storage therefore
produce a value denoting one declared variant; there is no payload ownership or
partial-initialization state to observe.

### Type boundary

The checker validates the enum declaration and every variant's zero-field
shape before admitting the operation. It compares nominal type identities,
not widths or layout identities. Two distinct two-variant enums remain
incomparable even when both lower to `i1`.

### Lowering boundary

The backend compares the two validated discriminants in the representation
already selected for `T`, for every permitted cardinality. A zero-variant enum
has no source-reachable value pair; a one-variant enum has only the diagonal
pair; neither case changes the operation domain or requires a new
representation rule.

`eeq` lowers to equality and `ene` to inequality. After normal operand
evaluation, neither primitive changes a tag, accesses a payload or memory,
allocates, traps, or creates an optimizer fact. Reading an operand through a
borrow still exhibits the ordinary EFF-2 read before the primitive executes.
The lowering is total for every source-reachable pair and introduces no poison
or undefined behavior.

### Invalid internal state

The operation relies on the same valid-discriminant invariant as `match`.
Malformed internal tags must be rejected at the producer/validation boundary;
the implementation must not treat `eeq` as a sanitizer for malformed compiler
state. Negative internal-state tests must exercise that boundary before the
operation ships.

## 6. Alternatives adjudication

| Alternative | Correctness | Performance/code shape | W1/regularity | Decision |
|---|---|---|---|---|
| v0.7 integer mappers | Only sound with a separate generated/reviewed injectivity guard; v0.7 exhaustiveness alone does not reject duplicate result codes | Extra source/helper structure; optimizer may recover caller compare | Large mechanical surface, exposed ordinal convention, and an external W3 guard | Conditional fallback |
| v0.7 nested pair match | Sound and accepted-language checked | No ordinal conversion, but branching source shape | 6,952 variant-pair arms for the 21 non-`Bool` types; existing Boolean operations at 11 `Bool` sites | Sound fallback |
| Widen `ieq`/`ine` | Can be implemented soundly | Direct compare | Violates truthful `i` prefix and one-domain spelling | Reject |
| Add `eeq`/`ene` | Sound under exact contract | Direct discriminant compare | New but predictable `e` domain; no overload | Recommend |
| Enum-to-int conversion | Can be made sound with more rules | Direct after conversion | Exposes representation and expands conversion surface | Reject |

The recommendation is not “change the spec because wfc violates it.” The
source census establishes demand and rewrite cost; the independent semantic
ground is that finite nominal tag identity is a total, representation-neutral
operation that the current language otherwise cannot express directly. The
W3 comparison is also material: direct `eeq` makes injectivity a checker and
lowering property, while the compact v0.7 mapper leaves it to an external
guard and the checker-closed v0.7 form costs 6,952 pair arms for the non-`Bool`
types.

## 7. Pre-approval hostile review gate

Before the exact delta is presented for owner approval, an independent hostile
review must attack at least these lenses and its verdict must be recorded:

1. **Type confusion:** same-width distinct enums, `Bool` versus a two-variant
   user enum, enum versus integer, struct versus enum, unresolved type IDs.
2. **Payload boundary:** every payload-carrying enum shape, including a mix of
   nullary and payload variants, must fail closed.
3. **Operation separation:** `ieq<Enum>`, `ieq<Bool>`, `eeq<Integer>`, and enum
   ordering must remain illegal.
4. **Discriminant validity:** malformed internal tags must not gain source
   validity through comparison.
5. **Effects:** both new primitives are exactly pure after normal operand
   evaluation; no primitive read, trap, or hidden helper call may be introduced,
   and ordinary EFF-2 reads needed to evaluate place operands must remain.
6. **Lowering parity:** `eeq` and `ene` must be complements at every supported
   tag width, including `i1`, without signedness-dependent behavior.
7. **Closed-world evolution:** adding an enum variant changes no equality call
   site and cannot make an old value compare equal to a different variant.

Any false accept or unclosed lowering edge is a no-go. The domain must not be
widened to make a test pass.

### Recorded pre-approval review result — 2026-07-19

The independent first pass returned NO-GO and found seven package defects: the
OP-8 text omitted zero- and one-variant enums; primitive purity was worded as if
it erased operand-read effects; the structural fallback count mixed the 21
non-`Bool` types with `Bool`; the exact conformance list was narrower than its
promised boundary; the OP-7 manifest wording referred to a not-yet-added case;
the stage-0 repair was sequenced before the source migration it would require;
and the decision-log entry followed rather than preceded its commit. Each was
corrected without widening the proposed operation domain.

A fresh read-only review of the corrected package returned **GO for presenting
the exact delta to the owner**. It independently reproduced the live
251-site/92-function/18-file/22-type census, the 21-type 262-arm and 6,952-pair
fallback arithmetic, and the in-memory 16,260 -> 16,670 token / 61,755 ->
63,571-byte materialization with 91 rules. It found the all-cardinality
semantics, operand-effect boundary, nominal/payload exclusions, direct selected-
representation lowering, 16-case conformance list, atomic migration order,
guarded metadata wording, and commit order coherent. It found no check
weakening, fact channel, conversion, ordering, payload equality, or unsupported
performance claim. This is a paper-design verdict, not the separate
implementation review required by Gate F. It does not turn the owner's earlier
scope-limited pre-approval into landing authorization; the exact companion
files in the delta still require explicit confirmation.

## 8. Exact landing gates if approved

### Gate A — baseline and approval

1. Run `make check` and `make -C compiler check` on the unchanged baseline.
2. Complete and record the pre-approval hostile review in §7.
3. Present the exact text in `V0.8-DELTA-DRAFT.md` to the owner.
4. Receive explicit approval for that exact numbered-spec and guarded companion
   delta. Plan or phase approval does not count.

### Gate B — specification coherence

1. Add `spec/kernel-spec-v0.8.md`; retain v0.7 as history.
2. Apply only the approved OP-1, OP-7, OP-8, header, ledger, live-reference,
   and conformance-META edits.
3. Add the documentary design-memory record specified by the delta draft:
   selected `eeq`/`ene`, with the widened-integer, source-mapper, structural
   pair-match, and enum/integer-conversion alternatives preserved with paired
   reasons. This records the decision and adds no semantics beyond the approved
   specification.
4. Only after those guarded edits exist, record the approved landing with
   `make approve-spec REASON="Owner-approved v0.7->v0.8 tag-only enum equality: exact eeq/ene delta and guarded conformance-META updates in enum-equality-investigation/V0.8-DELTA-DRAFT.md"`.
5. Require spec CI to report 91 unique rules and complete ledger coverage.
6. Require zero unknown cross-references, zero duplicate rule definitions, and
   zero new exception clauses.

### Gate C — stage-0 independence and exact typing

Gate C and Gate D items 1 and 3 through 5 are one atomic green implementation
slice. A standalone stage-0 repair would correctly reject the current wfc
unit's non-integer `ieq` calls before their migration and therefore cannot be
committed or treated as a completed step. Production lowering in Gate D item 2
may land in that slice or the next lowering slice, but it may not trust a CLEAN
classification as emission authority.

1. Teach the reference checker and democ the new operation names and exact
   tag-only domain.
2. Keep `ieq`/`ine` integer-only; add a regression that fails if they admit
   `Bool` or any user enum.
3. Lower `eeq`/`ene` directly at every tag width.
4. Add `eeq`/`ene` to the stage-0 FN-8 pure-total whitelist with exact tag-only
   type checking.
5. Add new reference tests rather than weakening or rewriting existing tests.
6. Obtain separate owner approval before changing any existing guarded
   reference test or frozen oracle digest.

### Gate D — production wfc semantics and source migration

1. Add exact operation recognition and type checking in wfc.
2. Add production lowering and re-verify the type/domain immediately before
   emission; a CLEAN classification alone is not lowering authority.
3. Re-run the investigation census against the live post-slice unit and record
   its delta from the 253-site/93-function/18-file/22-type baseline.
4. Mechanically migrate every live non-integer `ieq<T>` site to `eeq<T>`.
5. Re-run the census: zero non-integer `ieq`/`ine` sites and exactly the
   reviewed `eeq`/`ene` domain.
6. Repin the exact function, token, AST-node, CLEAN, and unsupported counts
   after helper growth.

### Gate E — conformance and negative canaries

Add new conformance cases for:

- three-variant `eeq` and `ene` truth tables;
- a zero-variant compile-only function and a one-variant truth table;
- `Bool` `eeq` and `ene` truth tables;
- distinct nominal enum rejection;
- payload-carrying enum rejection;
- integer operand rejection for `eeq`/`ene`;
- enum and `Bool` rejection for `ieq`/`ine`;
- enum ordering rejection; and
- explicit-type-argument and arity failures;
- a positive `requires` use; and
- representative integer and payload-enum `requires` negatives.

No existing expected verdict may be weakened. The three existing META coverage
annotations and the OP-7 description change only under the exact logged owner
approval enumerated in the delta draft.

### Gate F — second hostile review of the implementation

After the pre-approval paper review in §7, run a separate fresh hostile review
against the implemented checker and lowering. Include adversarial generated
programs across:

- zero-, one-, two-, three-, and many-variant tag-only enums;
- every operand equality/inequality pair;
- same-width distinct nominal types;
- payload and mixed-payload enums;
- `Bool` and two-variant user-enum `i1` lowering; and
- malformed internal symbol/type/tag metadata.

Every reproduced defect gets the smallest practical automated regression
before closure.

### Gate G — repository and phase resumption

1. Run `make check` and `make -C compiler check` after each completed slice.
2. Inspect raw IR and optimized assembly for the direct-compare property; make
   no timing or byte-size claim without a preregistered measurement.
3. Append one decision-log entry and make one commit per completed step.
4. Update THE-PLAN's execution cursor only after the accepted-language/source
   conflict is closed.
5. Resume the enum-element F1 slice only after stage 0 and wfc agree on the new
   operation domain and all hostile canaries pass.

## 9. Stop conditions

Stop and return to investigation on any of these outcomes:

- owner declines or narrows the exact delta;
- spec guard is red without matching logged approval;
- `ieq`/`ine` becomes legal for any non-integer type;
- `eeq`/`ene` accepts a primitive, payload enum, or distinct nominal type;
- the implementation requires exposing an enum ordinal to source;
- stage 0 and wfc disagree on any conformance case;
- raw or optimized lowering is not complement-correct for `eeq`/`ene`;
- hostile review finds an invalid-discriminant or type-confusion channel; or
- either repository gate is red.

## 10. Meaning of approval

Approval means only that the exact v0.8 text and guarded companion edits in
`V0.8-DELTA-DRAFT.md` may be landed through the logged governance workflow,
followed by the gated implementation above. It is not approval for payload
enum equality, enum ordering, implicit conversion, structural equality,
generic `eq`, a family reorder, or any work beyond the authorized roadmap.
