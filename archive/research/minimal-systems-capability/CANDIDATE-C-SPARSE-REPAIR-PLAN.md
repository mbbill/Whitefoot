# Candidate C Sparse Repair Plan

Date: 2026-07-15

Status: controlling paper-repair contract. The owner authorized route 1 after
the Hashbrown Gate 1 result: design and compare bounded repairs for the six
known Candidate C sparse-definition gaps. This authorization ends at the Sparse
Repair Gate defined below.

## 1. Objective

Determine the smallest finite Candidate C repair that gives exact paper routes
to the five already-audited Hashbrown operations without project-name
recognition, writer-defined proof authority, a new container family, or any
initialization, zeroing, payload traffic, allocation, metadata, indirection,
check, scan, synchronization, machine event, code-size, or asymptotic cost
absent from the pinned reference mechanism.

This pass must compare materially different repairs rather than elaborate one
preferred design in isolation. It selects at most a further-research
hypothesis, never a production language design.

## 2. Frozen evidence and scope

The only workload evidence is the committed Hashbrown v0.17.1 five-operation
audit and its 18 rows. Do not inspect more Hashbrown operations or another
project. The frozen operations remain:

1. lookup;
2. vacant insertion;
3. replacement;
4. removal; and
5. rehash.

The six gaps to repair are fixed:

1. C-4 control-metadata-to-payload admission;
2. C-4/C-6 sparse mutation, rehash, failure, abandonment, and cleanup;
3. C-4/C-7 entry/result provenance and invalidation;
4. C-11 sparse occupancy, group, probe, relocation, and rehash facts;
5. one exact C0-10 portable group-operation row and fallback; and
6. one exact C0-7 growth-allocation, failure, transfer, and release row.

The Candidate C v0 baseline remains immutable evidence. Proposed repairs go in
new documents and receive no authority until a later owner decision.

## 3. Frozen alternatives

Derive exactly three alternatives:

- `SR-CLOSED`: operation-closed sparse profiles. Each admitted sparse
  representation owns an explicit closed operation and fact catalog.
- `SR-PROFILE`: one profile-indexed sparse automaton. A closed descriptor schema
  selects control classes, slot relation, phases, transitions, facts, and access
  tokens from finite compiler-known tables.
- `SR-ORTHOGONAL`: factor sparse storage into generic checked relation,
  transition, provenance, and fact components. This is the compression control
  and must be rejected as a Candidate C repair if it silently becomes Candidate
  B's open composition algebra or Candidate A's general proof authority.

Do not add, rename, merge, or replace an alternative after scoring starts.

All three share two separately identified exact machine-leaf proposals. A
candidate cannot hide a missing C0 row inside an in-memory family rule.

## 4. Required definition for each alternative

Each alternative must state:

1. its finite public and compiler-owned vocabulary;
2. admissible representation profiles and who selects them;
3. control-state classes and exact payload-liveness relation;
4. normal and transition phases;
5. lookup, insertion, replacement, removal, resize, and in-place-rehash rules;
6. normal, precommit failure, partial-progress, abort, abandonment, and cleanup
   owner disposition;
7. vacant, occupied, bucket, entry, and result provenance;
8. fact producers, consumers, transfer, and invalidators;
9. exact C0-7 and C0-10 dependencies;
10. checker, compiler, backend, runtime, diagnostic, and AI-writer shape;
11. all eleven structural-cost dimensions;
12. feature-growth behavior and convergence toward B or A;
13. open problems; and
14. at least one observable falsifier.

Paper pseudocode may define state machines, but it is neither syntax nor an
implementation proposal.

## 5. Comparison rubric

Score every candidate-operation pair in a 15-row matrix. Each row must record:

- exact route and required state;
- owner traffic and cleanup;
- provenance and fact behavior;
- reference structural events;
- forced structural delta;
- safety status;
- boundedness and family-growth risk;
- B/A convergence risk;
- unresolved questions; and
- one observable falsifier.

Use exactly one route state:

- `CLOSED`: the proposed paper rules give a finite exact route.
- `OPEN`: a named rule remains undefined.
- `TAXED`: the route adds a structural event absent from the reference.
- `CONVERGES-B`: the route depends on an open orthogonal composition algebra.
- `CONVERGES-A`: the route depends on general writer-defined proof authority.
- `UNKNOWN`: the bounded evidence cannot decide.

`OPEN`, `TAXED`, `CONVERGES-B`, `CONVERGES-A`, and `UNKNOWN` prevent selection as
the Candidate C repair. `CLOSED` is a paper classification only, not a safety,
derivability, code-shape, or performance result.

Compare the alternatives across at least these dimensions: semantic surface,
number of compiler-known tables, representation freedom, reusable transition
coverage, provenance precision, cleanup precision, fact-schema count, checker
state, compiler/backend specialization, diagnostic locality, writer-visible
complexity, AI generation stability, structural no-tax result, new-family
pressure, cross-family matrix growth, and convergence toward B or A.

## 6. Structural and safety invariants

The Stage 0 ten universal invariants remain binding. In addition:

1. A normal vacant marker never authorizes a payload; any transition-local
   marker that authorizes one is confined to a non-escapable phase token.
2. Physical write ordering may differ from logical commit ordering, but no
   intermediate observation can produce contradictory control and liveness
   authority.
3. A sparse profile contains no library, project, API, path, symbol, key type,
   hash function, probing policy, or container identity.
4. Hash and equality calls never create occupancy or provenance authority.
5. Facts add no runtime metadata unless that metadata already belongs to the
   chosen reference representation.
6. The group-operation row gives behaviorally identical masks for every listed
   scalar and SIMD lowering and never reads beyond its checked control region.
7. The allocation row creates raw vacant storage only; payload liveness begins
   solely through the sparse transition rule.
8. A repair cannot call a persistent branch, field, tag, scan, allocation,
   atomic, fence, or machine event "proof-only" if it survives at runtime.

Any invariant violation disqualifies the alternative rather than becoming a
trade-off score.

## 7. Work limits and non-goals

- Candidate derivation: at most 60 minutes of semantic analysis.
- Comparison and hostile self-review: at most 30 minutes.
- Exactly three alternatives and fifteen candidate-operation rows.
- Reuse the existing audit; do not widen source inspection.
- Record `UNKNOWN` or `OPEN` when the time box expires.

This plan does not authorize:

- editing Candidate C v0, the language, syntax, specification, checker,
  compiler, backend, runtime, standard library, or containers;
- a formal safety model, mechanized proof, prototype, candidate construction,
  execution, generated-code check, benchmark, performance claim, or AI trial;
- a new family admission;
- an allocator, SQLite, Crossbeam, Tokio, Wasmtime, or other project audit;
- Stage 2 of the prior plan; or
- B compression work beyond measuring whether one alternative converges to B.

## 8. Deliverables

Produce and commit:

1. this controlling plan;
2. `CANDIDATE-C-SPARSE-REPAIR-CANDIDATES.md`, containing the three exact
   alternatives, the two shared C0 leaf proposals, the comparison, hostile
   review, and the Sparse Repair Gate result;
3. `CANDIDATE-C-SPARSE-REPAIR-MATRIX.tsv`, containing exactly 15 rows; and
4. a deterministic verifier for candidate identity, operation coverage, route
   vocabulary, row count, required fields, and one fail-closed gate result.

## 9. Sparse Repair Gate

Stop after the deliverables and choose exactly one disposition:

- `SPARSE-SELECT`: one alternative is `CLOSED` for all five operations, violates
  no invariant, has no identified structural tax, and remains distinctly C;
  name it as a further-research hypothesis only.
- `SPARSE-REVISE`: one or more finite repairs remain plausible but no frozen
  alternative closes the slice without an open rule, tax, or convergence.
- `SPARSE-NONE`: the slice falsifies bounded Candidate C repair because every
  alternative requires project recognition, unbounded family growth, general
  proof authority, open composition, or structural tax.

The gate authorizes no implementation or further audit. Any next step requires
another owner decision.
