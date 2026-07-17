# Performance-First Candidate Comparison

Date: 2026-07-15

Status: uniform paper comparison; no winner or production design is selected.

## 1. How to read the comparison

The companion TSV compares the three candidate sets over 17 dimensions. Every candidate cell contains an advantage, a liability, and the resulting trade-off. Every row states why the dimension matters, the current evidence level, and an observable decision rule or falsifier.

No numerical score is used. A total score would conceal three asymmetries:

1. One soundness failure is disqualifying and cannot be averaged against convenience.
2. One unavoidable tax on a protected weaker shape can eliminate a candidate for that route even if other routes are fast.
3. Paper generality, structural event counting, measured code shape, and measured application performance are different evidence classes.

## 2. Candidate A: proof-indexed resource calculus

### Pros

- It gives ordinary checked libraries the strongest autonomy. A new storage topology, ownership protocol, logical identity rule, or shared-lifecycle policy need not become a compiler feature if it reduces to the fixed logic.
- It has the best theoretical representation freedom. Proofs and indices may erase while each library stores only metadata required by its own contract.
- It offers one conceptual account of liveness, place authority, provenance, versions, cleanup, and interference.
- It is the least likely to need a keyword or family addition for every future performance problem.

### Cons

- The fixed logic is not small in the relevant sense. Finite maps, bounded quantification, provenance products, versions, cleanup, and happens-before must compose predictably and terminate.
- Proof production and diagnostics may be unsuitable for AI-written code, especially lower-tier first-green work.
- Proof-directed destruction and cleanup synthesis are underdefined and may create code-size or compile-time blowups even when runtime proof data erases.
- A small verifier kernel does not make the elaborator, state transfer, fact invalidation, or generated cleanup easy to review.

### Governing trade-off

Candidate A exchanges language-family growth for proof-system power. It wins only if a bounded prototype shows normal-frontend checking, stable proof size, local diagnostics, exact cleanup, and no protected-route code-size tax. Without that evidence, its generality is a liability rather than a demonstrated minimum.

## 3. Candidate B: bounded place-and-topology kernel

### Pros

- It directly matches the recurring performance needs: tag-free storage, representation-specific topology, direct owner traffic, scoped disjoint access, exact closure, and closed facts.
- It keeps named containers and algorithms in ordinary libraries while bounding the checker to table-driven forms.
- It makes no-tax distinctions explicit: dense versus sparse, append-only versus recyclable, unique versus shared, movable versus pinned.
- Its errors can name one missing topology, footprint, transition, lifecycle, or cleanup form, which is promising for AI repair.
- It provides the clearest path to structural accounting because every form has an explicit runtime representation and transition table.

### Cons

- Completeness rests on an unproved claim that the closed forms are orthogonal enough. Sparse control/payload coherence, concurrent migration, recursive partial cleanup, and address-sensitive retained state are likely stress points.
- The algebra can grow by accretion. Each new form adds checker, fact, cleanup, diagnostic, and interaction obligations.
- Fixed shared-lifecycle forms may accidentally encode policy that should remain in libraries.
- It offers less ordinary-library freedom than A and less per-family tuning than C.

### Governing trade-off

Candidate B exchanges some semantic generality for bounded implementation and review. It is the current structural middle hypothesis, not the current winner. It wins only if visible and held-out derivations use the existing forms without new trusted routes, hidden metadata, pathological traffic, or repeated algebra expansion.

## 4. Candidate C: family-specialized substrate

### Pros

- Each admitted shape has the most concrete representation, transition, cleanup, provenance, and lowering contract.
- Local checker rules and diagnostics are simpler than a general proof or orthogonal composition system.
- The compiler can pin exact optimized bodies and target-specific implementations for known operations.
- A family can be hostile-reviewed and structurally costed in isolation before wider use.

### Cons

- It has the largest language, compiler, backend, test, and teaching surface.
- It is most vulnerable to witness fitting: a new efficient topology or protocol normally requires a language addition.
- Ordinary libraries can wrap families but cannot innovate below the admitted family operations.
- Cross-family composition can duplicate metadata, create conversions, and grow a quadratic rule matrix.
- AI may complete local APIs easily while choosing a globally costly family architecture.

### Governing trade-off

Candidate C exchanges extensibility and compactness for local predictability. It wins only if a finite family set remains stable under independently selected held-outs and produces materially better checker simplicity or code shape than B. Repeated family additions or conversion costs falsify its minimum claim.

## 5. Pairwise distinctions

### A versus B

Both can erase proof authority and keep representation metadata selective. The difference is who may define a new state relation. A permits an ordinary module to do so through the fixed logic. B permits only composition of compiler-known topology, footprint, lifecycle, and cleanup forms. Therefore the decisive evidence is not a microbenchmark; it is whether held-outs require relations outside B and whether A can check those relations within the frontend, source-size, diagnostic, and code-size budgets.

### B versus C

Both use closed checker knowledge. B closes orthogonal low-level forms while leaving operations and containers to ordinary code. C closes whole family operations. Therefore the decisive evidence is whether B's ordinary implementations achieve the same structural events and optimized bodies without multiplying proof state, and whether C's family specialization materially simplifies checking or improves code enough to justify its larger surface.

### A versus C

They occupy opposite ends of the extension boundary. A resolves new demands through proofs; C resolves them through compiler-known families. A risks a proof language that is too powerful and difficult. C risks a systems language that grows one feature per admitted shape. If B fails, the reason for failure determines which direction remains plausible: missing expressiveness favors A, while checker or code-shape unpredictability favors C.

## 6. Current evidence-constrained ordering

There is no evidence-backed winner yet.

- B is the most plausible first validation hypothesis because its fixed transitions align with the observed storage, movement, footprint, cleanup, and no-tax obligations while avoiding A's general proof burden and C's family breadth.
- A remains necessary as the generality control. It becomes favored if independent held-outs repeatedly need relations outside B but those relations admit small deterministic proofs.
- C remains necessary as the specialization control. It becomes favored if B's composition creates checks, metadata, code growth, or checker complexity that fixed family operations avoid materially.

This ordering is a validation priority, not a recommendation. All three remain blocked by unmodeled safety, incomplete structural derivations, unenumerated machine-event tables, no AI evidence, and no candidate-specific measured performance.

## 7. Evidence that would change the ordering

- Promote A if it passes a bounded safety model, normal-frontend checker prototype, proof-size and compile-time ceilings, lower-tier AI construction, and no-tax structural gates while B needs repeated new forms.
- Promote B if the unchanged algebra derives all visible and independent held-outs, preserves exact budgets, passes model checking, and produces optimized bodies and AI repair behavior comparable to or better than the alternatives.
- Promote C if a stable finite family set covers independent held-outs and measured family lowerings or checker simplicity materially beat B without cross-family taxes or repeated additions.
- Select no winner if all three require either unsafe authority, unbounded rule/proof growth, hidden runtime tax, privileged named containers, or workload exclusions.

The next step is exact ordinary-library derivation and structural cost accounting against visible witnesses, protected baselines, and held-outs. No candidate execution or benchmark is authorized.
