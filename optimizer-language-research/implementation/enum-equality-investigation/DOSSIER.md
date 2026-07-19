# Enum equality investigation dossier

Status: **OPEN — NOT APPROVED, NOT APPLIED**

Date: 2026-07-19

This dossier investigates a conflict between the accepted language and the
current compiler source. It is evidence for an owner decision. It does not
change the kernel specification, checker, compiler, conformance expectations,
or execution order.

## 1. Question

Whitefoot v0.7 defines `ieq` and `ine` only for integer `T` [OP-1]. The wfc
source nevertheless uses `ieq<T>` where `T` is `Bool` or a user-defined
tag-only enum. The proposed F1 reader extension would have admitted the same
non-spec operation. The immediate question is therefore not how to implement
enum equality, but which of these two governing outcomes is justified:

1. keep v0.7 unchanged and rewrite wfc using operations already in v0.7; or
2. add a separately named, exactly typed equality family for tag-only enums in
   a new, owner-approved kernel-spec version.

The accepted specification is authoritative. Existing wfc source and stage-0
behavior are cost evidence, not authorization to widen OP-1.

## 2. Authority conflict

The relevant accepted text is the v0.7 OP-1 row:

> `ieq` `ine` `ilt` `ile` `igt` `ige` | all int T |
> `(T, T) -> own Bool` | pure

OP-7 also assigns the `i` prefix to the integer domain. Consequently:

- `ieq<Bool>` and `ine<Bool>` are outside the operation table;
- `ieq<MyTagEnum>` and `ine<MyTagEnum>` are outside the operation table;
- adding enum operands to `ieq`/`ine` would make the `i` prefix dishonest and
  make one spelling cover two domains; and
- stage 0 accepting such a call is a stage-0/spec discrepancy, not precedent.

The Phase-2 stop rule applies until the source is rewritten or a new spec
version is explicitly approved and landed.

## 3. Current-source census

A syntax-and-declaration census over `compiler/src/*.wf` found:

| Measure | Result |
|---|---:|
| Non-integer `ieq<T>` sites | 253 |
| Functions containing at least one site | 93 |
| Source files containing at least one site | 18 |
| Distinct tag-only operand types, including `Bool` | 22 |

These numbers are the investigation baseline, not a promise about the live unit
after intervening v0.7 Phase-2 slices. Re-run the same census immediately before
an approved landing and reconcile every changed site, function, file, and type
count in the landing record.

The intervening v0.7 tag-only-enum value slice replaced two internal
`ieq<AstKind>` calls with an already accepted exhaustive match helper. The live
post-slice census is therefore 251 sites in 92 functions and 18 files, over the
same 22 tag-only types. This live count is still evidence only; an approved
landing must re-run it after any further source change.

There is no corresponding non-integer `ine<T>` use in the current wfc unit;
the need for `ene` follows from a complete equality/inequality operation pair,
not from current-source frequency alone.

This census establishes that the conflict is structural rather than a single
mistyped call. It does not establish that the specification should change.

## 4. Cost of keeping v0.7 unchanged

Under v0.7, a nominal tag-only enum has no operation that exposes or compares
its discriminant. A source-only fallback must map each variant to an integer
and compare the two mapped integers. Projecting that rewrite over the current
wfc unit gives this static instruction census:

| Rewrite item | Projected cost |
|---|---:|
| Per-type mapper functions | 21 |
| Exhaustive mapper arms | 262 |
| Mapper calls at equality sites | 484 |
| Added canonical source lines | approximately 1,367 |
| Fully structural nested-match variant-pair arms for the 21 non-`Bool` types | 6,952 |
| Existing `Bool` equality sites handled with v0.7 Boolean operations | 11 |

The 21 mappers cover the non-`Bool` tag-only types. `Bool` can be compared using
existing boolean operations, but that is still a multi-operation spelling
rather than one equality operation. The line count is a mechanical projection,
not an applied patch and not a claim about generated tokens.

The compact mapper rewrite is syntactically legal and keeps the specification
unchanged, but v0.7 does not prove that its handwritten integer result codes
are injective. If two distinct variants receive the same code, integer equality
silently reports them equal. Exhaustiveness proves that every variant has an
arm; it does not prove that every arm returns a distinct integer. Shipping the
mapper form therefore needs a separate generated/reviewed injectivity guard,
and that guard is outside the ordinary v0.7 source judgment.

A fully structural v0.7 fallback can avoid result codes by exhaustively matching
the pair of variants and returning `True()` only on the diagonal. Projected
across the 21 non-`Bool` types, that form needs 6,952 variant-pair arms; the 11
`Bool` sites can use existing Boolean operations. It is checker-visible and
sound under existing exhaustiveness rules, but is a much larger source
expansion than the mapper projection.

The v0.7 fallback costs are therefore:

- two user-function calls at most enum-equality sites;
- one exhaustive arm per declared variant in each mapper;
- a duplicated ordinal convention that the language does not otherwise expose;
- a W3 injectivity obligation that the accepted checker does not discharge, or
  6,952 structural pair arms to avoid that obligation;
- more source and more call/effect checking in the compiler bootstrap; and
- a maintenance edit whenever a tag-only enum gains a variant, in addition to
  the exhaustive matches that already enumerate that enum for semantic work.

The rewrite remains a viable outcome because legality outranks convenience.

## 5. Focused code-shape probe

A temporary three-variant program was compiled with
`prototype/democ/democ.py`, then its LLVM output was compiled with
`/usr/bin/clang -O2 -S`.

The direct tag comparison lowered to one raw-IR `icmp eq i32`. The v0.7 mapper
shape lowered in the caller to two helper calls, two stack temporaries
(`alloca`/`store`/`load`), and one `icmp eq i64`, plus a three-arm branching
helper definition. Clang `-O2` recovered one comparison in the caller but
retained the helper definition.

This probe demonstrates a code-shape difference. It does not provide runtime
timings, byte counts, a whole-wfc measurement, or evidence that every backend
will make the same optimization choice.

## 6. Alternatives

### A. Keep v0.7 and rewrite wfc with exhaustive mappers

Form A1: each tag-only type gets a pure function that maps its variants to a
claimed-unique integer; equality maps both operands and uses integer
`ieq`/`ine`. A separate generated/reviewed guard must prove injectivity.

Form A2: match both enum operands structurally and enumerate every variant
pair, returning `True()` only for identical variants. This needs 6,952 arms
across the 21 non-`Bool` types in the current source projection; the 11 `Bool`
sites use existing Boolean operations. The form carries no unchecked ordinal
convention.

Advantages:

- no kernel-spec change;
- both forms use only already accepted constructs;
- A1 makes the variant-to-ordinal choice explicit in ordinary source;
- A2 avoids an ordinal convention and is checker-closed by exhaustiveness; and
- needs no new primitive checker or backend rule.

Costs and risks:

- the measured source expansion above;
- A1 is not W3-closed without a separate injectivity guard, because duplicate
  result codes silently equate distinct variants;
- A2 is W3-closed but expands to 6,952 variant-pair arms;
- helper-call and control-flow pressure before optimization;
- a mechanical ordinal convention repeated across 21 types;
- additional functions enlarge the exact self-hosting unit and its pinned
  census; and
- source shape expresses representation plumbing rather than the semantic
  question “do these values denote the same variant?”

### B. Widen `ieq` and `ine` to integers plus tag-only enums

Form: keep existing source spellings and change the OP-1 domain.

Advantages:

- smallest migration diff for current wfc;
- direct discriminant comparison; and
- no extra operation names.

Costs and risks:

- violates OP-7 because `i` is the integer-domain prefix;
- makes one spelling cover unrelated nominal and numeric domains;
- weakens the one-operation/one-domain regularity that lets a writer predict
  legality from the name; and
- turns existing invalid source into the design center of the rule.

This alternative is rejected by the dossier.

### C. Add `eeq` and `ene` for tag-only enums, including `Bool`

Form:

```text
eeq<T>(left, right) -> own Bool
ene<T>(left, right) -> own Bool
```

where `T` is one exact nominal tag-only enum type. `Bool` is included because
it is the canonical two-variant tag-only enum.

Advantages:

- the `e` prefix truthfully names the enum domain;
- integer, float, Bool-logic, and enum comparison remain distinct operation
  families;
- exact same-type checking prevents cross-enum comparison;
- direct lowering is one discriminant comparison;
- no ordinal mapping is exposed to source; and
- the migration is mechanical: replace only non-integer `ieq<T>` sites with
  `eeq<T>`; no current non-integer `ine<T>` site needs migration.

Costs and risks:

- two new writer-visible operation names and one new domain prefix;
- stage 0, wfc semantics, lowering, diagnostics, conformance, and teaching
  material all require coordinated support;
- a specification version bump and owner-gated guarded changes are mandatory;
  and
- weak-writer evidence for the `e` prefix has not been run independently.

### D. Expose or reinterpret enum discriminants as integers

Form: add or infer an enum-to-integer conversion and retain integer equality.

Advantages:

- reuses integer comparison.

Costs and risks:

- exposes a representation choice as writer semantics;
- creates an additional conversion operation and proof surface;
- invites integer operations and ordering on nominal tags;
- violates TYPE-4 if implicit and expands the spec more than direct equality if
  explicit; and
- weakens the compiler's freedom to change discriminant width.

This alternative is rejected by the dossier.

## 7. Tag-only soundness argument

The recommended domain is deliberately narrower than “all enums.” A tag-only
enum has no payload in any variant and is Copy under OWN-1. Every accepted value
therefore denotes exactly one declared variant of exactly one nominal enum
type.

For operands `a : T` and `b : T`, define:

- `eeq<T>(a, b)` is `True()` exactly when `a` and `b` denote the same declared
  variant of `T`; and
- `ene<T>(a, b)` is the exact boolean complement.

The checker requires both operands and the explicit type argument to resolve
to the same nominal `T`. It rejects integer operands, distinct enum types,
payload-carrying enums, structs, and every ordering operation over enums.

The lowering compares the validated discriminants in T's selected
representation. After normal operand evaluation, the primitive cannot create
an invalid tag, inspect a payload, move an affine value, access memory, trap, or
introduce undefined behavior. It is therefore pure and total. Reading an
operand through a borrow still exhibits the ordinary EFF-2 read before the
primitive executes; the `pure` operation row does not erase that effect. `Bool`
requires no exception: it is a tag-only enum, and equality is the same
same-variant predicate over its two valid values.

The soundness argument relies on the existing invariant that accepted enum
values have valid discriminants. Backend and malformed-internal-state tests
must attack that producer boundary; `eeq` itself must not turn an invalid
internal tag into an accepted source value.

## 8. Recommendation

Recommend alternative C: add `eeq` and `ene` for exact same-type tag-only enums,
including `Bool`, in a new v0.8 specification.

The recommendation is evidence-selected on two independent grounds:

1. the current-source census and projected v0.7 rewrite cost establish a real,
   repeated semantic need rather than convenience at one site; and
2. the focused code-shape probe shows that the direct operation maps to the
   machine comparison while the source mapper introduces recoverable but real
   helper structure.

This is not a safety relaxation. It adds a total operation over an already
closed, valid value domain. It also does not authorize its own landing. The
exact proposed text, owner gate, and negative canaries are in `PACKET.md` and
`V0.8-DELTA-DRAFT.md`.

## 9. Evidence limits

- The source projection was not implemented and does not measure compile time.
- The IR probe has no runtime timing or byte-count claim.
- No weak-writer naming trial compares `eeq`/`ene` with alternatives.
- No production wfc semantic or lowering path implements the proposed ops.
- No conformance case currently establishes the new behavior.
- Payload-carrying enum equality and enum ordering remain outside the proposal.

Any failed hostile review, cross-type false accept, payload false accept,
stage-0/wfc verdict difference, or codegen discrepancy returns the decision to
investigation rather than widening the domain.
