# Performance-First Capability Research — Owner Decision Packet

Date: 2026-07-15

Status: research complete for owner review; no candidate, validation, language change, or production work is selected or authorized.

## 1. Executive result

The research corrected the question from privilege isolation to performance expressiveness and produced three materially different capability hypotheses:

- Candidate A: proof-indexed resource calculus;
- Candidate B: bounded place-and-topology kernel; and
- Candidate C: closed family-specialized substrate.

All three share eleven abilities that do not arise merely from changing an ordinary representation: erased sealing, stateful direct callables, selective immovability, refinement, verified facts, failure/commit algebra, allocation roots, thread/atomic events plus a memory model, exact external events, exact target/device events, and executable-image lifecycle.

There is no evidence-backed winner. Candidate B is the first repair-and-validation priority, not a recommended language design. Its public Sparse rule, persistent cross-root provenance, and exact cleanup/abandonment rules are incomplete. Candidate A remains the generality control; Candidate C remains the specialization control.

## 2. What was delivered

1. A finite 15-row performance-expressiveness gap catalog routing all 43 non-established-or-protected capability obligations while retaining B-FIX and B-P2.
2. Three complete capability hypotheses with 44 distinct items, exact semantic intent, combination rules, deletion witnesses, library derivations, implementation shapes, safety invariants, open failures, and falsifiers.
3. A uniform 17-dimension A/B/C pros-cons comparison with no numerical score.
4. A 14-case, 42-route post-freeze derivability and structural-cost audit over protected controls, visible witnesses, and held-out contracts, with no paper PASS claims.
5. A 22-attack hostile review.
6. Six separately authorizable validation requests spanning semantic definition, safety, structural cost, machine events, performance, and AI stability.

None of these is a formal safety proof, structural artifact, candidate implementation, held-out execution, benchmark result, or production decision.

## 3. Why the result is performance-first

The first gap family is safe affine partial storage: the language must represent spare capacity or a hole without constructing T, reading it, or dropping it. That removes initialization, zeroing, sentinel, tag, and scan costs.

The audit shows why that single ability is insufficient. A high-performance ordinary library also needs direct owner traffic, exact commit/failure behavior, hole closure, result provenance, disjoint access, cleanup, and verified facts. Otherwise the program pays through swaps, rebuilds, copies, allocation, per-slot metadata, alias guards, repeated validation, or rejection.

Isolation is absent from candidate ranking. It appears only in C0-9 through C0-11, where ordinary code cannot be allowed to invent syscall, opcode, ABI, device, or loaded-image semantics. That is a consequence of exact machine behavior, not the research objective.

## 4. Candidate disposition recommended to the owner

| Candidate | Recommended disposition | Reason |
|---|---|---|
| A | Retain as generality control; do not select. | It can state the hardest relations but has no feasibility, checker, cleanup, AI, or cost evidence. |
| B | Retain as first repair-and-validation hypothesis; do not select. | It best matches recurring performance operations, but H-STORE, W-ARENA, W-RECUR, and W-PIPE expose blocking definition gaps. |
| C | Retain as specialization control; do not select. | It gives concrete local shapes but risks named-family privilege, cross-family tax, and uncontrolled language/backend growth. |

The recommended research hypothesis is:

> A bounded orthogonal kernel may be the smallest safe high-performance substrate only if its missing sparse, provenance, and cleanup rules can be made finite and public without acquiring A's general proof power or C's family-specific operations.

This is falsifiable. Convergence toward A or C rejects B as an independent minimum, even if the resulting A-like or C-like design remains worth studying.

## 5. Decisions requested

### Decision 1 — performance gap definition

- Accept the 15 conjunctive demand families as the current finite research frontier.
- Request specified revisions.
- Reject the frontier and state which workload or cost contract is wrong.

Acceptance does not accept any candidate mechanism.

### Decision 2 — candidate retention

For each of A, B, and C independently:

- retain for further research;
- request revision; or
- drop, with the condition that makes it unnecessary or dominated.

Recommended: retain all three in their control roles.

### Decision 3 — current conclusion

- Accept “no winner; B first repair-and-validation priority.”
- Request a different validation priority with reasons.
- Select no further research.

This decision does not select B as a language design.

### Decision 4 — validation authorization

Choose each request independently from `PERFORMANCE-FIRST-VALIDATION-REQUESTS.tsv`:

- VR-0 exact semantic repair;
- VR-1 deterministic safety model, only after VR-0 PASS;
- VR-2 structural cost artifacts, only after VR-1 PASS;
- VR-3 one exact machine-event model, independently scoped;
- VR-4 performance measurement, only after VR-1 and VR-2 PASS; and
- VR-5 AI stability, independently after paper rules freeze.

Recommended: authorize VR-0 only. Revisit VR-1 after hostile review of the repaired rules. Do not authorize VR-2, VR-4, or VR-5 yet. VR-3 can be authorized independently if the owner wants machine-boundary progress, but it does not help select A/B/C's storage substrate.

### Decision 5 — production progression

- Keep syntax, specification, checker, compiler, backend, runtime, standard library, fact-channel, migration, benchmark, and default-teaching work unauthorized.
- Or separately authorize an explicitly bounded later step after its prerequisites pass.

Recommended: keep all production progression unauthorized.

## 6. Risks that remain even if B repairs cleanly

- the bounded algebra may still fail future independent structures;
- checker seams may admit invalid state despite local rule simplicity;
- proof-derived facts may be stale or fail to produce the intended code shape;
- cleanup may be safe but code-size or compile-time expensive;
- AI may fail to select and repair the forms reliably;
- shared concurrency and memory reclamation remain unselected;
- external, target, and executable-image rows remain unenumerated;
- exact D-2 retains 340 unresolved obligations across 150 contexts;
- P-1 remains unmeasured.

## 7. What would justify a later design recommendation

A later packet may recommend a capability set only after:

1. exact paper definitions close with no convergence ambiguity;
2. hostile-reviewed safety modeling passes;
3. protected controls and structural witnesses pass exact event accounting;
4. ordinary checked libraries use only the public substrate;
5. candidate-specific code shape and performance match preregistered predictions;
6. AI construction and repair are stable under a frozen protocol; and
7. the applicable machine-event tables are enumerated for the claimed scope.

Until then, “no winner” is the accurate result.

## 8. Explicit non-authorization

This packet authorizes no syntax or keyword, kernel-spec change, checker rule, prototype/democ or xlc change, codegen/runtime/target/provider work, standard library, container, candidate, model, witness, held-out, benchmark, experiment, AI trial, E0.1 restart, migration, production fact channel, or default teaching.
