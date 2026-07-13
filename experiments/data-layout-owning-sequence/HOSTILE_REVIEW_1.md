# Hostile review 1: detached E0.1a prototype

Date: 2026-07-13.  Scope: post-correctness/code-shape review before any scored
timing or production implementation.  The reviewers made no code changes.

Reviewed artifact: detached worktree `/private/tmp/xlang-e01a-candidate` at
parent `58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`, dirty patch SHA-256
`bed070414f9552ea105857404d6d1296b98542a28cc65fa6899a197830e6774e`.
The patch changes only candidate checker/codegen tests and implementation
(`+968/-21` across four files); it has no feature flag.

## Verdict

The prototype establishes that 64-bit target-stride AoS field lowering and
unchanged-source raw-IR identity are feasible.  It does not establish a sound
record-buffer language design.  Two semantic blockers reject this artifact as
a production candidate; three major gaps prevent claiming complete E0.1a
surface or target support.  No performance conclusion exists.

## Evidence that passed

- complete candidate `make check`: `ALL VERIFICATION LAYERS GREEN`;
- checker 73/73 and 10,000-case modelcheck with zero reported soundness
  violations;
- all four frozen full-compiler/SoA facts-on/off raw-IR pins;
- 259 existing sources, including 135 baseline-success cases, with zero
  acceptance or IR-hash change under facts on/off;
- 14 old buffer signatures, including float and tag-enum cases, byte-identical
  between baseline and candidate;
- Flat facts-on/off native runs and independent ASan/UBSan runs (leak detection
  disabled because the disposable backend does not emit frees);
- TokenRow/AstRow target stride 24/56 on x86_64 and AArch64;
- field-only hot paths contain row GEP, field GEP, scalar load/store, and no
  aggregate load/store, `memcpy`, `byval`, or `sret`.

These passes did not exercise the paths below.

## Blockers

### B1 — `Flat != Copy` is bypassed by record fill

Candidate checker lines around `prototype/checker/checker.py:1237` accept both
`buffer_new<Record>(n, move seed)` and a fresh outer constructor containing a
nested affine move.  Candidate codegen around
`prototype/democ/democ.py:1265` and `:1320` evaluates/loads that aggregate once,
then stores the same value in every loop iteration.

Flat's no-borrow/no-drop layout proof makes the bit copy memory-safe for the
current narrow leaves; it does not license contraction of one nominal affine
value into N values.  Future private constructors or capability tokens make the
violation especially direct.  Explicit `move` consumes the source once; it does
not grant Repeat/Clone.

Required disposition: preserve the current `buffer_new<T>` T:Copy rule.  The
next narrow candidate may accept only a recursively fresh, no-`move` row recipe
with Copy leaves and per-slot construction semantics.  An explicit Repeat/Clone
capability is a separate owner decision and cannot be derived from Flat.

### B2 — index atoms bypass ownership flow

The ownership checker keys an indexed place by its container root and does not
walk the index atom as a value operand.  A moved affine record field can
therefore be used later as `index<Row>(rows, moved.value).field`; the checker
accepts it and codegen emits IR.  This is a pre-existing checker omission made
directly reachable by Flat field projection.

Required disposition: ownership/type-check every place operand, including index
atoms, exactly once and add hostile use-after-move tests before another
candidate is judged.

## Major findings

1. **Match scrutinee bypass.**  Whole-row `use`/`move` rejection lives in the
   ordinary expression path, while the match-scrutinee path calls place typing
   directly.  `match move index<Row>(...)` is checker-accepted and fails only in
   codegen.  Every expression context must fail closed in the checker.
2. **32-bit target unsoundness.**  Allocation uses i64 GEP/size arithmetic,
   `i64::MAX`, `malloc(i64)`, and a hand-written 64-bit `_size_align` model.  On
   i386, `Row { u64, u8 }` is really size 12/align 4 while the prototype emits
   `dereferenceable(16) align 8`; a request just above 4 GiB can truncate to an
   8-byte `size_t` argument and then be overrun by the fill loop.  The frozen
   x86_64/AArch64 experiment remains usable, but production must derive stride,
   ABI facts, pointer-index width, allocator size type, and `isize::MAX` from the
   selected DataLayout.
3. **Nested surface projection gap.**  Canonical
   `index<Outer>(...).inner.value` is tokenized as one `inner.value` field.  A
   noncanonical spacing workaround proves the nested GEP backend works, but the
   normative source path is not end-to-end supported.

## Benchmark-protocol blockers

1. `F-SOA-P` changes coallocation, pages/TLB, alignment, pointer provenance,
   alias information, and lifetime.  It is a composite diagnostic, not an
   allocation-count control, and no subtraction involving it supports a causal
   attribution.
2. Fixed versus sequence and reserve-exact versus doubling change owner/API,
   checks, destruction, capacity trajectory, allocator calls/size classes, and
   bytes moved.  They are total storage-policy comparisons, not isolated
   initialization/growth effects.
3. The former “two non-toy wins” rule permits endpoint cherry-picking.  A scored
   run needs a frozen endpoint family, single primary memory metric, ratio
   direction, confidence-bound benefit/NI rules, and multiplicity handling.
4. Cold, lifecycle/destruction, and retained lanes need separate and equivalent
   owner lifetime.  Retained samples may not inherit capacity across corpora;
   construction/warm-up cost remains reported.
5. The equal 64 MiB Darwin virtual stack reservation is acceptable experiment
   infrastructure, but static frame and measured high-water must be hard
   guardrails and production migration must pass the target's default stack
   contract.

## Residual gates

No malloc fault injection, MSan, target-default stack run, complete 32/64-bit
DataLayout matrix, machine-code identity pin, internal tape-by-tape equivalence,
or lifecycle/free validation exists.  The current backend retains allocations,
so its process-exit behavior cannot prove STOR-3 or retained-memory semantics.

Next stop: owner review of initialization semantics and the corrected protocol.
No blocker may be converted into implementation work in the production
toolchain without explicit owner confirmation.
