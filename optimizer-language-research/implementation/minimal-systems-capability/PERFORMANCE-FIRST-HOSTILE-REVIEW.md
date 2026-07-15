# Performance-First Candidate Hostile Review

Date: 2026-07-15

Verdict: no candidate is selectable as a language design; Candidate B may be retained only as the first repair-and-validation hypothesis, with A and C retained as controls.

Status: hostile paper review only. No model, prototype, candidate, witness, held-out, benchmark, AI trial, specification change, or production implementation was constructed or run.

## 1. Review rule

The review asks whether a candidate can remove the registered performance barriers while preserving every universal acceptance invariant and protected weaker-shape budget. It does not ask whether a candidate hides dangerous abilities from ordinary users. Safe ordinary-library expression is a benefit. Isolation matters only for exact machine semantics that ordinary code cannot create.

One soundness failure, hidden machine event, protected-route tax, workload exclusion, or privileged named-container route is disqualifying. No advantage can average it away.

## 2. Attack matrix

| ID | Attack | Candidate A | Candidate B | Candidate C | Result |
|---|---|---|---|---|---|
| HR-01 | Hidden full initialization, zeroing, sentinel T, Option tag, or bitmap enters partial storage. | Proof-indexed liveness can avoid it, but erasure is untested. | B-1/B-2 explicitly separate storage from T, but Sparse representation rules are incomplete. | Dense/Hole families avoid it locally; cross-family wrappers remain untested. | OPEN for all; structural artifact required. |
| HR-02 | A universal descriptor, header, generation, refcount, pin, target flag, or image state taxes weaker shapes. | Indices should erase, but proof plumbing and specialization may tax code size. | Forms are separate by design; generic widening or conversion may import stronger state. | Families are separate; shared helpers and wrappers may duplicate or widen headers. | OPEN; B-FIX and B-P2 exact identity required. |
| HR-03 | Replace, removal, growth, or migration hides Clone, Default, extra swap traffic, whole-buffer copy, or second persistent payload copy. | A-4/A-5 distinguish owner traffic, but lowering is absent. | B-3 distinguishes each transition directly. | Family operations specify traffic locally, but inconsistent family definitions are possible. | PAPER MITIGATION only; owner-event trace required. |
| HR-04 | Recoverable allocation or callback failure occurs after destructive commit and loses offered owners or requires an uncharged rollback copy. | Pre/post states can express it; proof-directed rollback is undefined. | B-4/C0-6 give commit scopes; recursive/callback-dependent cases remain open. | Each family can freeze commit order; duplication across families is a risk. | OPEN; exhaustive failure injection and state model required. |
| HR-05 | An affine transition value is abandoned with a live hole, retained authority, or pending cleanup. | A-6 states exact-use/cleanup but synthesis is unproved. | B-4/B-9 state closure but W-RECUR/W-PIPE exceed current detail. | Family cleanup can close known cases but cross-family abandonment is incomplete. | BLOCKING for all. |
| HR-06 | A returned or stored borrow derives from the wrong owner, registry slot, token address, temporary view, or call frame. | A-7 can state exact roots; W-ARENA transfer is unproved. | B-6 records roots; token-relocation preservation is not exact. | C-7 records family results; persistent multi-root placement is underdefined. | BLOCKING W-ARENA provenance gap. |
| HR-07 | Dynamic disjointness is validated once but remains usable after mutation, version change, relocation, or interference. | Versioned propositions can invalidate; interaction proof is large. | B-5 scopes versions; complete invalidator table is absent. | Family invalidators are local and may disagree across composition. | OPEN; stale-version negative canaries required. |
| HR-08 | Sparse metadata claims a payload is live without a public exact control-to-place relation. | A-2 can state H-STORE's relation, subject to proof feasibility. | B's public Sparse admission/verification rule is missing. | C-4 can encode it only if it is a generic substrate, not recognized H-STORE. | Candidate B BLOCKED; Candidate C definition risk. |
| HR-09 | Dense and append-only paths pay sparse occupancy or identity-reuse costs. | Library representations can differ; proof/code tax untested. | Closed forms separate them explicitly. | Separate families can pin them, but wrapper conversions can reintroduce costs. | OPEN no-tax evidence. |
| HR-10 | Shared ownership or concurrency adds refcount, fence, lock, lease, header, or indirection to unique code. | A states separate indexed states; memory model feasibility is unproved. | B-8 separates lifecycle forms; policy may be baked in. | C-10 separates families; compiler/runtime surface is broad. | OPEN; unique-path exact-event control required. |
| HR-11 | A weak-memory or reclamation rule permits race, stale payload access, premature release, ABA, or double destruction. | General happens-before proofs are the hardest A interaction. | C0-8/B-8 tables are only categories, not a selected memory model. | C-10 families likewise lack exact model and reclamation semantics. | BLOCKING for all concurrent claims. |
| HR-12 | Selective pinning silently becomes universal heap allocation or indirection, or logical identity is confused with address stability. | C0-3 separates states, but no lowering exists. | C0-3/B forms separate handle and address state. | C-9/C-10 have explicit families, with highest conversion risk. | OPEN; address-sensitive structural control required. |
| HR-13 | Stateful callable effects, invocation counts, result provenance, or cleanup are guessed, enabling call elision/reordering or owner loss. | A-5 can express exact partitions, at high proof cost. | C0-2/B-6 give fixed relations; W-PIPE resurrection and cached state remain complex. | C0-2/C-7 freeze adapter cases; surface and cross-products grow. | BLOCKING W-PIPE definition gap. |
| HR-14 | Refinement validation is copied, repeated, permanently tagged, or remains valid after mutation/foreign replacement. | A/C0 facts can bind exact roots and versions. | B-10 fixed schema can do so if every invalidator is listed. | C-8/C-11 can do so family by family, risking inconsistency. | OPEN; fact lifecycle artifact required. |
| HR-15 | Proof or state authority becomes writer-forgeable through assertion, unchecked contract, custom axiom, plugin, cached artifact, or optimizer hint. | A has the largest attack surface; fixed kernel and no new axioms are mandatory. | B's closed forms reduce the surface. | C's fixed operations reduce fact forgery but enlarge compiler identity surface. | PAPER MITIGATION; hostile producer/verifier tests required. |
| HR-16 | External I/O/FFI uses a generic syscall/symbol/ABI descriptor, hides staging copies or duplicate calls, or loses partial progress/resources. | Shared C0-9 is unenumerated. | Shared C0-9 is unenumerated. | Shared C0-9 is unenumerated. | BLOCKING for all general-systems completeness claims. |
| HR-17 | Target/device operations use arbitrary opcodes/descriptors, scalarize, insert helpers/dispatch/copies, or omit ordering/reset semantics. | Shared C0-10 is unenumerated. | Shared C0-10 is unenumerated. | Shared C0-10 is unenumerated. | BLOCKING for all target completeness claims. |
| HR-18 | Generated code is proved pre-link but the loaded image differs, W^X overlaps, cache events are missing, or unload races execution. | Shared C0-11 is only a table form. | Shared C0-11 is only a table form. | Shared C0-11 is only a table form. | BLOCKING for executable-image claims. |
| HR-19 | Proofs, topology combinations, cleanup, adapters, or family variants grow compile time or reachable code superlinearly. | Highest proof-normalization and cleanup risk. | Medium combination and monomorphization risk. | Highest family/backend breadth and cross-product risk. | OPEN; preregistered build and code-size ceilings required. |
| HR-20 | AI requires nonlocal proofs, chooses the wrong form/family, repairs by copying/allocating, or cannot interpret diagnostics. | Highest expected proof-generation risk. | Best bounded-pattern hypothesis, with no evidence. | Easiest local API, highest global family-selection risk. | OPEN; separate AI trial required. |
| HR-21 | Candidate coverage uses a privileged named container, recognizable source name, hidden standard module, or candidate-private helper. | General proof route should avoid it, but proof libraries may become de facto privilege. | Orthogonal forms should avoid it; Sparse admission is where a private predicate could enter. | Highest risk because compiler-known families can drift into named containers. | BLOCKING anti-special-casing review. |
| HR-22 | Future gaps cause uncontrolled extension: new axioms for A, new forms for B, or new families/operations for C. | Infrequent but very powerful logic changes. | Accretion can erase the claimed middle position. | New family is the default repair and therefore the greatest growth risk. | OPEN; removal witnesses and convergence test required. |

## 3. Candidate verdicts

### Candidate A — retain as generality control; do not select

A remains the only candidate that can state H-STORE's relation without adding a new compiler-known sparse encoding. That is real value. It is not evidence that A is minimal or feasible. Its fixed logic, proof terms, cleanup synthesis, cross-root provenance, happens-before reasoning, diagnostics, compile time, and AI construction are all unvalidated. A is rejected for selection and retained to test whether B's missing relations genuinely require general proof power.

### Candidate B — repair-and-validation priority; do not select

B remains the most plausible bounded route for partial storage, direct affine traffic, scoped disjointness, and no-tax separation. It is not a complete capability set. The public Sparse relation, persistent multi-root provenance, and exact cleanup/abandonment rules are blocking definition gaps. B survives only as a hypothesis: repair those rules without a general predicate/proof language and without adding family-specific operations. If repair moves either way, B has converged toward A or C and loses its independent rationale.

### Candidate C — retain as specialization control; do not select

C gives the most concrete local representation and lowering story. It is not currently minimal. Its public sparse and access families may disguise held-out/container recognition; cross-family composition and duplicated metadata are unaccounted; the language/compiler/backend surface and future growth are largest. C is retained to determine whether specialization materially improves code shape or checker simplicity over B.

## 4. Recommendation

There is no winner and no capability set should be selected.

The evidence-backed recommendation is to retain all three hypotheses and authorize, if the owner chooses, only the smallest definition-and-safety work on B while using A and C as explicit convergence controls. This recommendation is about research order, not language design.

B remains an independent candidate only if its repaired rules satisfy all of the following:

1. H-STORE's public sparse relation is derivable by unrelated ordinary libraries without an arbitrary predicate, private proof rule, scan, bitmap tax on dense paths, or container recognition.
2. W-ARENA's payload borrows remain rooted in exact backing allocations across owner-token relocation with no borrow table, per-placement check, universal pin, or indirection.
3. W-RECUR and W-PIPE cleanup is exact, bounded, and code-size stable without user finalizers or privileged adapters.
4. B-FIX and B-P2 remain exact historical controls.
5. The repairs add neither A-style general proof authority nor C-style family-specific operations.

Failure of item 5 is not necessarily failure of the project. It is evidence selecting the next comparison branch: A if general relations are necessary and feasible, C if specialization is necessary and bounded.

## 5. Evidence that changes the recommendation

- Favor A only after a fixed-logic safety model, normal-frontend proof/checking prototype, bounded proof/compile/code size, exact cleanup, no-tax artifacts, and AI stability pass.
- Favor B only after the three definition gaps close, the unchanged algebra derives the full audit set, safety and structural gates pass, and candidate-specific code shape and AI evidence are competitive.
- Favor C only after a stable finite family set survives independent new witnesses and materially improves checker simplicity or code shape over B without recognition, cross-family tax, or repeated additions.
- Continue with no winner if every repair requires unsafe authority, hidden runtime cost, unbounded rule growth, privileged named containers, or workload exclusion.

Exact D-2, P-1, concurrency, external, target, and final-image obligations remain fail-closed.
