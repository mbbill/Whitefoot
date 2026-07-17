# Performance-First Derivability and Structural-Cost Audit

Date: 2026-07-15

Status: post-candidate-freeze paper audit; no witness or held-out implementation, source, test, trace, candidate execution, or measurement.

## 1. Freeze and evidence boundary

The candidate definitions were committed first as `f5d6c0d`, with packet hash `8f729e7f1654e760df2aa93e9b493c4593973f3b2ed5292f134e5135fe3c8761`, before this exact contract-by-contract audit. This is not a blind held-out validation: the candidate research already knew the held-out identities, dependency roles, and structural ceilings through the gap and witness inputs. Candidate source, tests, traces, performance observations, thresholds, and manual tuning remain excluded. The held-outs therefore test post-freeze derivability and anti-special-casing only; they provide no statistical or independent-generalization credit.

The companion TSV covers the two protected baselines, eight visible witnesses, and four held-outs. Each A/B/C cell states an ordinary checked-library route, the expected structural account, and an evidence status. The matrix deliberately has no PASS status.

The status vocabulary is:

- `CONTROL_CLAIM_UNTESTED`: the candidate promises not to affect a protected baseline, but there is no candidate artifact.
- `ROUTED_UNPROVED`: the paper route and cost shape are explicit; safety and structural artifacts are absent.
- `ROUTED_DEFINITION_RISK`: the route exists only if a named semantic point is made exact.
- `HELDOUT_PAPER_ROUTE`: a post-freeze contract route exists without source, trace, or execution.
- `HELDOUT_DEFINITION_GAP`: the candidate lacks an exact public rule needed by the held-out.
- `HELDOUT_DEFINITION_RISK`: a route is plausible but may collapse into recognition or an over-narrow representation.
- `HELDOUT_CONDITIONAL_PREDECESSORS`: the composition route cannot receive credit before its imported families close.

## 2. Main findings

### 2.1 All no-tax claims remain untested

A, B, and C can all describe a way to leave B-FIX and B-P2 unchanged. That is not evidence. The controls require exact historical source, diagnostics, layout, raw IR, optimized body, checks, calls, facts, and counters. A proof that a feature is semantically unused is insufficient if its compiler path or generated code changes.

### 2.2 Partial storage is necessary but not sufficient

W-SMALL, W-GAP, H-FLATSET, and W-ECS all need dead/live place distinctions and direct affine traffic. They also need exact commit ordering, failure owner return, overlap direction, abandonment closure, result provenance, and cleanup. A single uninitialized-storage keyword would remove forced initialization but would not close these contracts.

This is the clearest confirmation of the owner's framing: safe uninitialized storage is a performance ability, but the minimal set must also let ordinary code complete the high-performance transition without copies, tags, rebuilds, or invalid intermediate states.

### 2.3 H-STORE exposes B's most important definition gap

Candidate B says that a closed `Sparse` view relates control metadata to payload places. H-STORE requires the exact public relation

`position[k] = p AND p < len AND dense_key[p] = k AND valid_T(dense_value[p])`.

An unrelated library must establish, preserve, invalidate, and consume that relation across insert, remove, swap repair, iteration, and destruction. B currently does not say how a new dense/sparse-array encoding is admitted without a private predicate, compiler-recognized container, scan, or general proof system. Therefore B's 15/15 routing remains bookkeeping, not semantic closure.

A can state the relation in its fixed finite-map logic, subject to proof feasibility. C can put it in the public Sparse family, subject to proving that the family is a general storage substrate rather than a built-in H-STORE and that representation choices remain adequate.

### 2.4 W-ARENA exposes a shared provenance gap

All candidates must distinguish three identities:

1. the dense registry slot holding a sealed block-owner token;
2. the token value that may relocate when the registry grows; and
3. the backing allocation that physically roots payload borrows.

Moving (1) or (2) must preserve (3). A payload borrow may derive from the arena owner and exact backing root, never the registry slot or token address. None of the paper candidates yet gives a formal cross-call rule that proves this while later placement mutates a disjoint fresh place. This is not an address-stability requirement for every value and must not import universal pinning or indirection.

### 2.5 Cleanup remains a first-class capability question

W-RECUR and W-PIPE prevent the candidates from treating affinity as completion. Bounded-stack recursive destruction needs a non-fallible, exact owning worklist route. Traversal abandonment needs branch-specific callable and source disposition. A's proof-directed cleanup, B's fixed cleanup forms, and C's family cleanup all remain underdefined at these edges.

### 2.6 The held-outs preserve the A/B/C distinction

- H-FLATSET is friendly to all three because its state is a dense prefix plus a scoped hole; it still rejects finished-container privilege and post-commit callbacks.
- H-STORE favors A's expressiveness, directly attacks B's closed sparse relation, and tests whether C's family is genuinely generic.
- H-LRU and H-IPQ do not favor a low-level mechanism yet. They test whether separately closed public families compose without new raw access, special hooks, allocation, scans, or ownership ambiguity.

## 3. Candidate-specific result

### Candidate A

A has a paper route for every witness and held-out contract because its fixed logic can state the required relations. It receives no feasibility credit. The audit increases concern about proof size, exact cleanup synthesis, cross-root provenance, and code growth. A remains the generality control.

### Candidate B

B gives the most direct visible routes for W-SMALL, W-GAP, W-ECS, and direct place traffic. It also exposes the sharpest exact gap: the public admission and verification rule for a new sparse control/payload relation. W-ARENA provenance and W-RECUR/W-PIPE cleanup remain definition risks. B remains the first validation priority only after those definitions are repaired without turning it into A or C.

### Candidate C

C has concrete routes through its fixed families, but the audit raises the anti-special-casing burden. C-4 must be a public sparse substrate, not a compiler-known map or H-STORE path; C-7 must support persistent multi-root fresh placement, not only call-local splits; family composition must not duplicate headers or require bespoke hooks. C remains the specialization control.

## 4. What is and is not established

Established by bookkeeping:

- all 14 controls/witnesses/held-outs have one A/B/C route and structural account;
- each cell names its unresolved proof or falsifier;
- no held-out source, trace, or performance result was used, but held-out identities and budgets were prior research inputs;
- no candidate was modified after opening the held-out contracts.

Not established:

- memory, race, ownership, provenance, cleanup, or optimizer soundness;
- exact D-2 closure or P-1 performance;
- any no-tax, allocation, traffic, code-size, compile-time, or throughput claim;
- any ordinary-library implementation;
- exact external, target, or image table completeness;
- candidate ranking beyond validation priority.

## 5. Consequence for the next step

The hostile review must not recommend B as a capability set in its present form. It may recommend B as the smallest hypothesis to repair and validate, with two explicit paper prerequisites:

1. define the fixed public Sparse admission relation sufficiently to derive H-STORE without a user-defined predicate, container recognition, or a universal bitmap; and
2. define persistent multi-root provenance and cleanup rules sufficiently for W-ARENA, W-RECUR, and W-PIPE.

If those repairs require a general proof logic, B converges toward A. If they require new family-specific operations, B converges toward C. That convergence behavior is itself a falsifier for B's claimed middle position.

No repair, model, prototype, witness construction, held-out execution, or benchmark is authorized by this audit.
