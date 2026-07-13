# E0.1 Research Report: Data Layout and Owning Sequences

Status: non-normative report for owner review, 2026-07-13. Isolated experiments are
allowed; production implementation, specification changes, xlc migration, scored
performance experiments, and default teaching are not authorized.

## 0. Executive conclusion

Specification and implementation audits, not performance experiments, confirm the
first real expressiveness gap: xlang can manually express SoA with multiple
`buffer<primitive>` values, but it cannot safely place an ordinary named record in a
`buffer<Record>` or guarantee that `index<Record>(rows, i).field` lowers correctly to
a row GEP, field GEP, and scalar load/store. Programs with an obviously row-oriented
producer and consumer are therefore forced into parallel arrays. This constrains
performance choices and scatters field additions and removals, length consistency,
capacity checks, and API boundaries across several buffers.

This does not mean that the current xlc should change from SoA to AoS. The existing
compiler has many consumers that scan only a kind, span, or child-link column. SoA's
unit stride, lack of record padding, and per-column alias scope may all be superior
for those consumers. Static accounting even shows that directly replacing only the
fixed Token/Ast storage with natural AoS under the same full-capacity policy would
increase requested memory for those two tapes by 11.1% and for the entire 30-column
frontend by 3.85%. The correct objective is therefore to let the language explicitly
express either layout while protecting the existing SoA from regressions, not to
assume one global winner.

The smallest design direction worth preserving is:

1. Do not add `copy struct`, `flat struct`, a layout attribute, or automatic layout
   selection.
2. Keep every record affine and do not broaden `ImplicitCopy(T)`.
3. Have the checker independently derive a `Flat(T)` predicate used only for storage
   eligibility.
4. Permit `buffer<FlatRecord>` to have fixed-length, fully initialized AoS storage
   and scalar field access, but first resolve how to initialize every row without
   copying an affine value.
5. `Flat` must not permit explicit or implicit contraction/cloning. Whole-row moves,
   observable padding, bytewise comparison or hashing, and ABI promises remain
   forbidden.
6. Do not use a feature flag. Baseline and candidate are always two isolated
   toolchains, each with one unconditional semantics.

The isolated prototype proved that two properties can hold together: on
x86_64/AArch64 it emits target-stride 24/56-byte rows whose field hot paths contain
only a row GEP, field GEP, and scalar load/store; it also keeps all four existing
SoA/full-compiler raw-IR pins byte-identical. Hostile review nevertheless rejected
that prototype as a production candidate: it copies one affine fill value into N
slots, and the checker fails to track ownership of index atoms. A green test suite
does not remove either blocker.

A second gap also exists: the current `buffer<T>`, whose entire length is initialized,
cannot express `{ptr, len, capacity}` with only `[0, len)` initialized at zero overhead.
Using `buffer<Option<T>>` or filling the entire capacity introduces tag, branch,
initialization, and drop costs. This must be decided separately as E0.1b. Implementing
it together with AoS would make it impossible to attribute effects to layout,
unused-capacity initialization, or growth policy.

This report does not request a production implementation. The first owner decision
needed now concerns record initialization semantics, not performance timing. Even a
later successful candidate would not imply that xlc should change layouts.

## 1. Scope

This phase answers only two questions:

- **E0.1a:** Does the language lack a way to express fixed-length AoS record storage
  without broadening implicit copying and without adding a runtime tax?
- **E0.1b:** Does the language lack a way to express an initialized prefix and
  capacity in an owning sequence without exposing uninitialized memory?

E0.1a must reach an adopt/reject/defer conclusion before E0.1b opens. Modules,
methods, contracts, borrowed aggregates, loop facts, byte literals/bulk append, and
SIMD belong to later E0.2-E0.5 work and must not be added to this candidate.

The experimental boundary is:

- Do not change the specification, checker, stage 0, xlc, or teaching semantics in
  the main worktree.
- Keep the candidate only in a detached disposable worktree, with its semantics
  applied unconditionally.
- Do not introduce a switch, dual grammar, or a path that co-locates baseline and
  candidate behavior.
- Static accounting, correctness tests, negative tests, IR/assembly shape checks,
  and unscored smoke runs are allowed.
- Formal timing requires a separately frozen protocol after protocol review.
- Production implementation requires explicit confirmation after review of this
  report.

## 2. What the current baseline establishes

The frozen baseline is Git `58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`. A full
`make check` passed before any candidate implementation: checker modelcheck covered
10,000 cases with 0 soundness violations, codegen parity had 0 failures, and
conformance had 231 passes/13 skips with 90/90 rules covered.

The fixed input for the canonical compiler unit is:

| Item | Value |
|---|---:|
| source bytes | 1,029,044 |
| source SHA-256 | `17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65` |
| tokens | 211,374 |
| AST nodes | 105,550 |
| per-column capacity | 1,029,045 |
| total request for 30 fixed columns | 214,041,360 B (204.13 MiB) |

The current three Token columns have widths `4 + 8 + 8 = 20 B/index`; natural AoS
stride is 24 B. The current seven Ast columns have widths `4 + 6 x 8 = 52 B/index`;
natural AoS stride is 56 B.

| Metric | Token + AST |
|---|---:|
| current fixed SoA request | 74,091,240 B (70.66 MiB) |
| fixed natural AoS request | 82,323,600 B (78.51 MiB) |
| semantic field bytes in the live prefixes | 9,716,080 B (9.27 MiB) |
| Token / AST capacity utilization | 20.54% / 10.26% |

These numbers expose three independent variables:

1. **Layout:** SoA has no row padding and gives a single-column consumer contiguous
   access. AoS makes whole-row producers and consumers contiguous and reduces
   repeated length/capacity operations across parallel columns.
2. **Allocation shape:** Token/Ast currently use ten field buffers, while AoS would
   use two allocations. Allocation count and layout locality must not be described
   as the same benefit.
3. **Initialization policy:** Every column is fully initialized to a capacity of
   `source_len + 1`, while only about 10%-21% is used. This may be the larger memory
   or cold-start problem, but it is orthogonal to AoS versus SoA.

A direct comparison of "current SoA versus growable AoS" would therefore bundle
three changes, and no result could support a language-design conclusion.

## 3. Actual performance and maintenance impact of the missing capabilities

### 3.1 Missing `buffer<Record>`

Performance constraints:

- A row-oriented lexer/parser producer must address, bounds/capacity-check, and
  write several allocations.
- An author cannot directly choose to share a cache line and address calculation
  among fields in a row.
- The language has no zero-cost representation for small records or whole-row
  traversal that is naturally suited to AoS.
- A library cannot encapsulate AoS as a reusable component and must instead repeat
  hand-written parallel buffers.

Maintenance constraints:

- Fields of one logical record are scattered across several globals/structure
  members and function parameters.
- Adding or removing a field requires synchronized changes to initialization,
  writes, capacity, borrows, and consumers.
- "Every column has the same length" remains a cross-object invariant rather than
  being encapsulated by one storage owner in the type system.
- Code review has difficulty distinguishing buffers that jointly form one tape
  from unrelated arrays.

The central issue is not record methods or inheritance; the data structure itself
cannot have the physical representation selected by its author. A future `impl`
could group the names of a hand-written SoA API, but it could not make AoS
expressible.

### 3.2 Missing initialized-prefix owning sequence

Performance constraints:

- A buffer preallocated for the maximum input must initialize the unused tail.
- `buffer<Option<T>>` adds per-slot state, branches, and drop scanning.
- A library cannot provide a no-growth push that is only a store plus `len += 1`
  after capacity has been reserved.
- Growth cannot express that elements have moved into a new allocation and the old
  block must be raw-freed without dropping them again.

Maintenance constraints:

- `buffer<T> + len` is merely a convention in a comment; existing buffer semantics
  still state that every slot is initialized.
- Reserve/grow behavior, borrow invalidation, failure atomicity, and exact-once
  dropping cannot be centralized in one opaque owner.
- Every library would reinvent an incomplete vector convention.

This is where a language/runtime substrate is genuinely required. It must not be
smuggled in by changing existing `buffer` semantics, because that would change the
initialization and destruction contract of default fixed-buffer programs.

## 4. E0.1a option comparison

| Option | Performance expressiveness | Maintainability | Principal risk | Assessment |
|---|---|---|---|---|
| A. Keep only hand-written SoA | No new cost | Parallel-column invariants stay exposed | The best layout for row-centric workloads remains inexpressible | Insufficient |
| B. Permit storage after making records structurally `Copy` | Can store AoS | Superficially simple | Bare uses, parameters, and returns may create invisible large copies | Reject |
| C. Separate `Flat(T)` from Copy and permit `buffer<Record>` | Preserves both SoA and AoS; byte-identical old code is a hard gate | Record storage has one owner | Initialization can evade OWN-1 if it repeats an affine fill | Preserve direction; reject current prototype |
| D. Add an `@soa`/`@aos` representation attribute | Can specify layout | Central declaration | Couples source records, storage layout, ABI, and field access; too large for the first step | Defer/reject for now |
| E. Let the compiler choose or convert automatically | Could theoretically select the winner | Less author code | Access patterns cross functions, choices are unstable, and conversions have hidden costs | Reject |

Option C does not require new declaration syntax for the storage choice.
`buffer<Record>` already explicitly selects a contiguous row per element, while
several hand-written primitive buffers still explicitly select SoA. The checker can
determine whether a record can safely occupy storage, but that layout property does
not determine how the record may be copied or initialized in bulk.

## 5. Precise definition of the recommended candidate

### 5.1 `Flat(T)` is not `ImplicitCopy(T)`

Candidate `Flat(T)` means only that the target can statically determine the size and
layout, that the value contains no region/borrow and no drop obligation, and that it
can serve as a fixed-storage element. It does not mean:

- a bare use copies the value;
- every bit pattern, zero, or padding is valid;
- `memcmp` or bytewise hashing/serialization is allowed;
- field offsets, ABI, or FFI representation are stable;
- a whole-record load/store is necessarily cheap.

For fields of a new record, the first prototype accepts only exactly represented
integer primitives, tag-only enums, and recursively eligible non-empty named records.
It conservatively rejects floats inside records, ZSTs, payload enums, arrays,
buffers/boxes/arenas/slices/cells/sequences, generics/erased/unknown types, finalizers,
and by-value cycles. These restrictions narrow the prototype; they are not permanent
judgments about the final language. Existing `buffer<f32>`/`buffer<f64>` uses remain
on their primitive path and must not be narrowed by the new predicate.

### 5.2 Fixed-buffer initialization is the current blocker

Existing `buffer_new<T>(n, fill)` permits only Copy T because its semantics repeat a
once-evaluated `fill` N times. The prototype incorrectly accepts
`buffer_new<Record>(n, move seed)`: the backend loads `seed` once and then stores it N
times in a loop. Although `Record(inner: move seed)` is a fresh top-level construct,
it still copies the nested affine value N times. Flat proves only that a bit-copy
will not double-free; it cannot prove that nominal affine state or a capability may
be copied semantically. Writing `move` explicitly does not make contraction legal.

The current recommendation for the next isolated experimental candidate is:

- Keep the existing T:Copy rule for `buffer_new<T>` unchanged.
- For records, accept only a recursively fresh row recipe.
- Recipe leaves may only be Copy scalars/tags, and the entire tree must contain no
  `move`, affine-producing call, borrow, or resource.
- Semantically construct each slot independently; the backend may reuse bits only
  under the as-if rule.
- Freeze the operation name, subexpression evaluation count, and effect rules only
  after owner review.

An explicit Repeat/Clone conformance is another possible route, but it is broader
than Flat storage and cannot be automatically derived, so it should be deferred. If
a fresh recipe is unacceptable, a dynamic-length record buffer needs a safe per-slot
builder or initialized-prefix owner. E0.1a must not pretend that initialization has
already been solved.

Once storage is fully initialized, allow scalar field reads/writes, `len`, existing
conservative borrowing, and replacement of an existing complete row with a fresh
construction. Continue to forbid bare whole-row reads and
`move index<Record>(...)`, which would leave a hole. Padding has no observable
language semantics.

### 5.3 Required codegen shape

The target `DataLayout` determines size, alignment, stride, and field offsets. Record
allocation must check `count x stride`, alignment rounding, the target pointer-index
or `isize` limit, and allocation failure. Checking only `u64` multiplication overflow
is insufficient to make LLVM `inbounds GEP` valid.

The hot path of `index<Row>(rows, i).field` must be:

1. A bounds check consistent with existing semantics.
2. A row GEP.
3. A field GEP.
4. A scalar load/store for the field.

Field paths must not produce whole-record materialization, aggregate loads/copies,
`memcpy`, `byval`, or `sret`, and cannot rely on later SROA to "perhaps fix it."
Existing primitive-buffer/SoA fixtures in the candidate toolchain must have
byte-identical raw LLVM, and optimized assembly/call/trap/alias shape must also be
unchanged.

The disposable backend currently satisfies this shape only on the frozen 64-bit
x86_64/AArch64 lane. It still hard-codes an i64 allocation/index limit and 64-bit
`_size_align` facts. On i386, `Row { u64, u8 }` really has size 12/alignment 4, while
the prototype declares `dereferenceable(16) align 8`; requests over 4 GiB can also be
truncated by 32-bit `malloc(size_t)` before writes continue. A production
implementation must derive stride, ABI alignment, pointer-index width, allocator
`size_t`, and `isize::MAX` from the target DataLayout. The current prototype cannot
claim to be target-generic.

### 5.4 Limited evidence from the isolated prototype

The prototype exists only in `/private/tmp/xlang-e01a-candidate`, has no feature
flag, and has not entered the production toolchain. It passed:

- Full `make check`, ending in `ALL VERIFICATION LAYERS GREEN`.
- Checker 73/73 and modelcheck with 10,000 cases / 0 soundness violations.
- Four full-compiler/SoA raw-IR identity pins, with facts both on and off.
- All 135 baseline-success cases among 259 existing sources, with no acceptance or
  IR-hash change with facts either on or off.
- Fourteen legacy buffer signatures, including f32/f64 and different tag enums,
  byte-identical to baseline.
- Native run, ASan, and UBSan for the Flat fixture with facts on and off.
- 64-bit TokenRow/AstRow strides of 24/56 and field-only paths with no aggregate
  copies.

These tests show that the layout/codegen direction is implementable; they do not
show that the candidate semantics are sound. The existing suite missed affine
contraction, index-atom use after move, a match whole-row bypass, 32-bit facts, and a
nested-field surface-parser problem. A green result therefore cannot justify
adoption.

## 6. Minimal semantic outline for E0.1b (implementation remains closed)

If E0.1a closes first, a later candidate should be an opaque affine `sequence<T>`,
not a change to `buffer<T>`:

- Logical state is `{data, len, capacity}`.
- Only `[0, len)` is initialized, readable, and subject to drop. The tail is
  inaccessible and has no per-slot tag or bitmap.
- Separate `reserve/grow` from `push_within_capacity`.
- A push with proven `len < capacity` must be only an element store and length
  increment.
- Reject any potentially growing operation while an element/slice borrow is live.
- After successful growth, do not drop moved elements in the old allocation; only
  raw-deallocate the old block.
- Allocation failure leaves the old sequence intact.
- Support only `Flat` elements initially; arbitrary affine elements wait for STOR-3
  exact-once drop.
- Provide `replace` initially; do not expose a hole-producing `take`, public
  `MaybeUninit`, or arbitrary partial initialization.

This makes the initialized prefix an internal invariant of the type/owner without
adding a capacity field, growth branch, drop loop, or runtime `needs_drop` test to an
ordinary fixed buffer.

## 7. Principal attack surfaces found by hostile review

1. **Flat bypasses affinity.** A moved fill, or a nested move inside a fresh wrapper,
   is evaluated once and written N times. This is a production blocker; the
   initialization semantics must change.
2. **Index atoms do not enter ownership flow.** A moved record field can still be
   used as an index, creating a real OWN-1 use after move. This is an old checker
   defect, but the new projection directly encounters it.
3. **The whole-row prohibition has expression-context gaps.** A match scrutinee
   bypasses the checker and fails only in codegen. The checker must fail closed in
   every context.
4. **32-bit DataLayout/facts are unsound.** The i64 limit and manually computed
   alignment suit only the current 64-bit experiment. Production needs
   target-derived allocator/index/facts behavior.
5. **Source syntax cannot parse recursive projections.** The tokenizer combines
   `inner.value` into one field in `index<Outer>(...).inner.value`; only a single
   record level is actually covered today.
6. **`F-SOA-P` is not a single-variable allocation-count control.** Coallocation
   also changes pages/TLB, alignment, pointer provenance, aliasing, and lifetime. It
   may be used only as a combined diagnostic, not for causal subtraction.
7. **F/R/D still change the entire owner policy.** Fixed-to-sequence changes header,
   API, checks, and drop behavior together. Reserve-to-doubling changes the capacity
   trajectory, reallocation, and bytes moved together. Report only end-to-end total
   effects.
8. **AoS padding can be overlooked.** Token grows from 20 to 24 bytes and Ast from
   52 to 56 bytes. Record requested, initialized, live, and touched bytes; RSS alone
   is insufficient.
9. **Producer time can conceal consumer behavior.** Measure single-column,
   mixed-field, child-link, whole-row, and complete-frontend workloads.
10. **Padding can leak into semantics.** Canonicalize results field by field; forbid
    raw record comparison or hashing.
11. **Field projection can take the wrong backend path.** Pin checker, typed-AST, and
    lowering behavior for `.field` after `index<T>`; do not first select the
    whole-element scalar fallback.
12. **Statistical rules can permit cherry-picking.** Freeze workload, phase, and
    memory metrics in advance. Gains must meet a conservative CI bound, while
    guardrails must use a preregistered non-inferiority margin.
13. **Cold and retained lifetimes can be unfairly compared.** Keep cold latency,
    lifecycle/destruction, and retained measurements as separate lanes, and do not
    carry capacity across corpus inputs.
14. **A feature flag creates two languages.** Rejected. Experiments use two isolated
    single-semantics toolchains.
15. **A capability experiment can be misreported as xlc migration.** These require
    different gates. A local AoS win cannot overturn SoA for column-heavy xlc.
16. **The default AI writer is a risk.** An expert selecting the right layout does
    not show that a low-tier writer will do so. Default teaching needs a separately
    authorized, benchmark-blind Terra writer panel.

An early attempt isolated the candidate with checker/codegen flags. Review correctly
identified that approach as violating a single canonical language, and all related
mainline code was fully withdrawn. The detached-worktree method now removes that
attack surface during experimentation instead of leaving it for production repair.

## 8. Experimental design and decision gates (scoring requires owner approval)

Recommended matrix:

| ID | Layout | Storage / initialization | Attribution purpose |
|---|---|---|---|
| F-SOA | Current SoA | fixed, full-capacity initialization | Sole production baseline |
| F-SOA-P | Separate columns, one tape allocation | same fixed/full initialization | Combined coallocation diagnostic; no causal subtraction |
| F-AOS | Flat-record AoS | same fixed/full initialization | Total representation effect versus F-SOA |
| R-SOA / R-AOS | One logical owner each | `reserve_exact`, initialized prefix | Total reserve-policy effect |
| D-SOA / D-AOS | One logical owner each | same doubling trajectory | Total doubling-policy effect |

E0.1a completes only F-* first. R/D belong to E0.1b and do not start automatically
because E0.1a performs well.

Hostile review withdrew the early threshold of "wins on at least two non-toy
workloads": two results could be selected after the fact from corpus x phase x scale
x microkernel. Before scoring, freeze a machine-readable endpoint registry that
specifies the complete primary family, candidate/baseline ratio direction, single
primary memory metric, maintenance edit tasks, non-inferiority margins, sample
counts, and multiple-comparison or hierarchical gate. A time or memory gain must
place the conservative upper bound of the 99% CI at `<=0.90` or `<=0.85`,
respectively, rather than relying on a point estimate or "not significant." Every
other primary endpoint and guardrail must pass its preregistered non-inferiority
bound. The exact workload set and margins still require owner review, so scoring
cannot begin now.

Even if the capability passes, xlc migration has a stricter gate. For both complete
compiler cold and retained lanes, the 99% CI upper bound for candidate/S0 must be
`<= 1.000`; lifecycle/destruction, phase, microkernel, scale, memory, and stack
guardrails must all pass preregistered non-inferiority bounds; and there must be a
material gain of at least 5% time, 15% memory, or 15% maintenance burden. Both arms
must use the same 64 MiB virtual stack, while static frame/high-water usage must also
be pinned so a large reservation cannot hide stack growth. Mere parity does not
justify migration risk.

Default teaching additionally requires an independent low-tier Terra panel and new
external-disclosure authorization. Previous percent-decode or utf8parse permission
cannot be reused. This phase will not send material to an external model.

## 9. Conclusions supported and not supported by current evidence

Supported:

- `buffer<Record>` is a real expressiveness gap, not merely xlc implementation debt.
- Flat storage must be separate from implicit Copy.
- Flat cannot automatically imply explicit Repeat/Clone either.
- The current SoA has plausible performance justification and must not be replaced
  based only on a maintenance intuition.
- An owning sequence is also missing, but it is orthogonal to layout.
- A feature flag is incompatible with one canonical language.
- The 64-bit field-GEP/scalar-lowering direction is implementable.
- Safe initialization, checker flow, and target-generic layout must be resolved
  before any performance claim.

Not supported:

- Fixed AoS is faster for xlc.
- The current detached candidate is semantically sound or production-ready.
- `buffer_new<Record>(n, move seed)` is legal.
- Benefits of an initialized-prefix sequence come from AoS.
- xlc should migrate to AoS or growable storage.
- `Flat` should enter the specification or production toolchain.
- A default AI writer will reliably choose the right layout for different access
  patterns.
- A sequence of arbitrary affine elements is already sound.

External research likewise shows only that layout depends on workload. SoAx reports
SoA advantages in particle workloads; another Lennard-Jones study finds AoS better
under a particular vectorization/padding scheme; and SoCal suggests that factored
multi-buffer layouts can be strong for compiler-like recursive data. These results
justify bidirectional microkernels and complete xlc measurements; they cannot replace
project data.

## 10. Decisions requested from the owner

1. Do you agree that safely expressing fixed `buffer<Record>` storage is a real
   E0.1a language-capability gap?
2. Do you agree to preserve a storage direction with no declaration marker and with
   `Flat(T) != ImplicitCopy(T)`, while explicitly stating that Flat also does not
   imply Repeat/Clone?
3. Should the next and only isolated prototype use a "recursively fresh, Copy leaves,
   no move anywhere in the construction tree" recipe for record initialization? This
   report recommends it; explicit Repeat/Clone remains a separate design.
4. Do you agree that the first version should continue to forbid whole-row
   extraction, observable padding, payload enums, ZSTs, floats inside records, and
   nested owning storage, and should first close the index-atom, match, and nested
   parser gates?
5. Do you agree to keep E0.1a and the owning sequence strictly serial, with no
   len/cap/growth behavior included in E0.1a?
6. Do you agree that the revised protocol must not be scored until the endpoint
   registry, lifetime lanes, and causal-control blockers are closed?
7. If the capability eventually passes, should xlc migration and default teaching
   still require separate approvals rather than treating capability adoption as
   authorization for either default migration?

Production implementation remains frozen until these questions are reviewed.

## References and local evidence

- Frozen local baseline: [`BASELINE.md`](BASELINE.md)
- Candidate semantics: [`FLAT_DESIGN_CANDIDATE.md`](FLAT_DESIGN_CANDIDATE.md)
- Full experimental draft: [`PROTOCOL.md`](PROTOCOL.md)
- Hostile review 0: [`HOSTILE_REVIEW_0.md`](HOSTILE_REVIEW_0.md)
- Hostile review 1: [`HOSTILE_REVIEW_1.md`](HOSTILE_REVIEW_1.md)
- Separation of external sources and inferences: [`RESEARCH.md`](RESEARCH.md)
- LLVM `getelementptr`: <https://llvm.org/docs/LangRef.html#getelementptr-instruction>
- LLVM aggregate code-shape guidance:
  <https://llvm.org/docs/Frontend/PerformanceTips.html#avoid-creating-values-of-aggregate-type>
- Rust `Vec` initialized-prefix guarantee:
  <https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees>
- Rust `Copy`: <https://doc.rust-lang.org/reference/special-types-and-traits.html#copy>
- Rust allocation `Layout`: <https://doc.rust-lang.org/std/alloc/struct.Layout.html>
- LLVM vectorizer: <https://llvm.org/docs/Vectorizers.html>
- SoAx: <https://arxiv.org/abs/1710.03462>
- Lennard-Jones AoS/SoA study: <https://arxiv.org/abs/1806.05713>
- SoCal: <https://arxiv.org/abs/2605.01140>
