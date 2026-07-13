# Hostile review 0: protocol and semantic decomposition

Date: 2026-07-13.  Phase: before prototype implementation.

Historical note: post-prototype `HOSTILE_REVIEW_1.md` overturns this memo's
provisional resolution of whole-record fill/duplication and its causal reading
of `F-SOA-P` and the F/R/D matrix.  This file preserves the pre-prototype attack
record; review 1 and the revised protocol control current decisions.

This memo records attacks that must be answered by the protocol or an
implementation artifact.  It is intentionally written against adoption.

## Resolved in the protocol

1. **Bundled-feature confound.**  A fixed AoS record, initialized-prefix storage,
   and growth can each change time and memory.  They are now serial decisions
   with fixed, reserve-exact, and doubling SoA/AoS controls.
2. **Allocation-count confound.**  AoS reduces Token/Ast allocations from ten to
   two and combines capacity checks.  `F-SOA-P` separately controls allocation
   headers; trap/check counts are reported rather than relabeled as locality.
3. **Padding hidden by an AoS claim.**  Current Token/Ast effective widths are
   20/52 bytes but natural AoS strides are 24/56.  Exact requested, initialized,
   live, and touched bytes are mandatory; RSS alone is rejected.
4. **Producer-only benchmark.**  The workload includes kind-only, span-only,
   child-link, mixed, and full-row consumers plus the complete frontend.
5. **Wrapper/allocator pollution.**  The current public wrapper retains
   allocations.  Each sample is a fresh native process; retained timing uses a
   dedicated fixture.
6. **Structural Copy creates invisible large copies.**  Aggregate flat storage
   is separated from implicit Copy.  Whole-record duplication is explicit and
   field projection must not materialize the aggregate.
7. **`buffer<T> + len` masquerades as Vec.**  A fully initialized fixed buffer
   cannot represent uninitialized spare capacity or raw-free moved storage
   without tags, fill, or double-drop hazards.  The sequence is an opaque
   initialized-prefix owner and does not mutate fixed-buffer semantics.
8. **Growth infects the hot path.**  Reserve/grow is separate from
   `push_within_capacity`; the proven no-grow form has an assembly-shape gate.
9. **Dropping moved elements twice.**  Retired backing storage uses internal raw
   deallocation, not ordinary recursive drop.  General affine elements wait for
   STOR-3; the first prototype is Flat-only.
10. **Old/default programs pay for a dormant feature.**  Unchanged fixtures must
    retain byte-identical raw IR and assembly and may not gain cap fields,
    growth/drop branches, runtime type tests, or runtime declarations.
11. **Unsafe allocation arithmetic.**  `u64` multiplication overflow is not
    enough for target `inbounds GEP`.  The gate also checks layout rounding,
    alignment, and the pointer-index/`isize` maximum.
12. **Result cherry-picks a new xlc default.**  Capability adoption, xlc layout
    migration, and default-writer teaching have separate thresholds.  A local
    row-centric AoS win cannot migrate column-centric compiler tapes.
13. **Affine row read creates a fixed-buffer hole.**  A whole-element
    `move index<Record>(...)` cannot be admitted merely because Record has no
    destructor: fixed storage promises every slot is initialized, while treating
    the move as a copy would silently enlarge Copy.  The first slice permits
    field projection/overwrite and rejects whole-row extraction.
14. **Post-index field is lost or mis-typed.**  The existing disposable backend
    was written for scalar elements; its typed-AST mapping and codegen dispatch
    can discard `.field` after `index<T>` or choose the whole-element path first.
    Tests must pin the post-field AST, exact container/`index<T>` agreement, and
    row-GEP then field-GEP lowering.
15. **A feature flag creates two languages.**  A default-off switch would avoid
    changing old output but still make one checker/compiler maintain two
    semantic universes, violating the canonical-language rule.  Any approved
    experiment must instead compare two isolated, single-semantics toolchains;
    no flag or dual grammar may enter the main tree.

## Boundary before any production E0.1a implementation

- Obtain owner review and explicit confirmation before touching production
  spec/checker/stage 0/xlc/teaching.
- Experimental candidate code uses a disposable branch/worktree; never add a
  flag or candidate semantics to the main checker/stage 0.

- Implement correct target size/alignment and aggregate buffer addressing in
  stage 0; a checker-only prototype is unsound.
- Freeze the exact `Flat`, explicit record-copy, padding, ABI, and failure
  semantics in a non-normative prototype note.
- Produce the unchanged-source raw-IR/assembly identity report.
- Produce field-only IR showing no aggregate load, `memcpy`, `byval`, or `sret`
  in the hot path.
- Implement the independent `F-SOA-P` diagnostic control.
- Freeze corpus generator seeds/hashes and the native phase-timing boundary.

## Blockers before E0.1b

- Close E0.1a with an explicit adopt/reject/defer record.
- Establish reborrow provenance for element and slice borrows so reserve/growth
  is rejected while any is live.
- Implement overflow/alignment/allocation-failure rules and a no-grow append
  with no allocator slow path.
- Decide whether allocation failure traps or returns a value; do not benchmark
  arms with different policies.
- Complete exact-once destruction before claiming support for non-Flat affine
  elements.  Flat-only sequence performance may be measured earlier, but it is
  not evidence for the general case.

## Required later attacks

Review 1 must inspect code, generated IR/assembly, sanitizers, cross-target
layout, and raw counters before scored timing.  Review 2 must recompute every
reported interval from raw logs, test schedule balance and exclusions, and
challenge the final claim boundary.  An author summary is not a substitute.
