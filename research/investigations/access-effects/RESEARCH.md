# Access effects: the ownership and reference redesign

Outcome: candidate x1, recorded in [CANDIDATE-X1.md](CANDIDATE-X1.md), is the owner-confirmed rule set for references, storage, effects, and overlapped execution. It replaces the current specification's borrow modes, regions, store brands, and providers with: references as local names for paths, never stored or returned; validity as a fact; effects only on reference parameters; a pairwise call-site check; one global heap with an affine `Box`; three storage shapes (`Array`, `Slots`, `Ring`) with a compiler-maintained window; value classes with `linear` kept; and overlap permission by path disjointness. No specification, compiler, or design-tree change is made here; the next step is a specification amendment written from CANDIDATE-X1.md.

## How the result was reached

1. Requirements were separated from mechanisms and settled by debate ([VERDICT-D0.md](VERDICT-D0.md)); the current mechanisms were mapped to them ([MECHANISM-MAP.md](MECHANISM-MAP.md), [EVIDENCE-mechanism-survey-2026-09-16.md](EVIDENCE-mechanism-survey-2026-09-16.md)). Finding: the current borrow apparatus serves interference facts and the optimizer only, and the current compiler emits no alias facts to LLVM.
2. Every option per decision point was enumerated ([OPTIONS.md](OPTIONS.md)) and discriminating programs written ([PROGRAMS.md](PROGRAMS.md), [CASES.md](CASES.md)).
3. A first core model (value semantics plus branded pools with ghost slot state) was selected and then found unreliable in prose derivation ([VERDICT-CORE.md](VERDICT-CORE.md), [DISPOSITIONS-CORE.md](DISPOSITIONS-CORE.md), [COMPARISON-CORE2.md](COMPARISON-CORE2.md)). Rejected.
4. Corpus and practice evidence: 7802 borrow sites in the existing corpus, none unwritable without stored references ([EVIDENCE-corpus-census-2026-09-16.md](EVIDENCE-corpus-census-2026-09-16.md), [EVIDENCE-boundary-surveys-2026-09-16.md](EVIDENCE-boundary-surveys-2026-09-16.md)).
5. In a parallel session the owner derived a candidate x0 with stored references, ran a 300-cell matrix over it ([MATRIX-X0.md](MATRIX-X0.md), [GAPS.md](GAPS.md), method and history in [DESIGN.md](DESIGN.md)), and abandoned stored references after the pool-provenance argument: an object whose pool cannot be determined statically must carry a pool pointer at run time.
6. The reference restriction that emerged was frozen as x1 and attacked: first on vector reallocation ([EVIDENCE-vector-growth-refute-2026-09-18.md](EVIDENCE-vector-growth-refute-2026-09-18.md)), then by a 136-cell matrix and eight engineering tasks ([matrix-x1/GAPS-X1.md](matrix-x1/GAPS-X1.md), reviewer re-check [REVIEW-X1-round1.md](REVIEW-X1-round1.md)), then by a targeted second round after the owner's rulings ([matrix-x1-r2/GAPS-X1-r2.md](matrix-x1-r2/GAPS-X1-r2.md), [REVIEW-X1-round2.md](REVIEW-X1-round2.md)). Every hole found was a wording defect closed by one sentence; the surviving costs are listed at the end of CANDIDATE-X1.md.

## Files

| File | Status |
|---|---|
| CANDIDATE-X1.md | Result. The frozen rule set, revision 5, no open proposal. |
| REVIEW-X1-round1.md, REVIEW-X1-round2.md | Reviewer's re-trace of the matrix findings against the rule text. |
| matrix-x1/GAPS-X1.md, matrix-x1-r2/GAPS-X1-r2.md | Judged reports of the two x1 rounds. The raw per-cell derivations, verifications, and task programs (about 2 MB) were removed from the tree in the cleanup and remain in history at commit 4ca62f8db758. |
| EVIDENCE-vector-growth-refute-2026-09-18.md | Adversarial check of the static-scope rule across reallocation. |
| VERDICT-D0.md, MECHANISM-MAP.md, OPTIONS.md, PROGRAMS.md, CASES.md | Requirements, mechanism map, option catalog, discriminating programs. Still valid. |
| EVIDENCE-mechanism-survey-2026-09-16.md, EVIDENCE-boundary-surveys-2026-09-16.md, EVIDENCE-corpus-census-2026-09-16.md | Survey and census data. |
| VERDICT-CORE.md, DISPOSITIONS-CORE.md, COMPARISON-CORE2.md | Rejected first core model and its critiques. |
| DESIGN.md, MATRIX-X0.md, GAPS.md | The abandoned x0 candidate (stored references), its matrix, and the method notes. |
| ../../experiments/access-state/ | Executable bounded model of the earlier alias/state fragment; superseded, kept as an experiment record. |

Removed in the cleanup and recoverable from history at 4ca62f8db758: the raw debate transcripts and option-enumeration outputs of 2026-09-16, the x0 cell files, the E1/E2 drafts (target packages, range families).

## What the specification amendment must do

Delete: `&uniq`, regions and region statements, store brands and providers (`Heap<'s>`, `Arena<'s, ...>`, `Box<'s, T>`, `Vector<'s, T>`), `Slice`/`MutSlice` as value types, returned reborrows, `allocates` effects, holder and child-reborrow machinery, capability-based linearity. Add: the path grammar with payload steps, validity facts, the pairwise call-site rule, `Array`/`Slots`/`Ring` with pseudo-field measures and window parts, the window operations, `swap` and the atomic update, consuming moves out of a field, `&[T]`, the allocator axiom. Keep: copy/affine/linear classes and the `linear` modifier, contracts and `ensures when`, entailment and `use` steps, PAR-1/PAR-2 restated as path overlap. Then rewrite the conformance corpus and hand the checker's facts to LLVM (noalias, captures(none), memory(argmem), scoped alias metadata, loop parallel accesses), which the current compiler does not do.
