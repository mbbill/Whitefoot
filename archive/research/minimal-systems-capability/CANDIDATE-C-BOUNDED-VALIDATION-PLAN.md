# Candidate C Bounded Validation Plan

Date: 2026-07-15

Status: controlling research contract. The owner authorizes only plan
durability, Stage 0, and Stage 1. Work must stop at Gate 1. Stage 2 and every
later stage require separate owner authorization.

## 1. Objective

Determine whether a finite Candidate C family set lets ordinary checked xlang
libraries express mature standard-library and high-performance systems
mechanisms without writer-accessible unsafe, project-name recognition, or
forced initialization, zeroing, copying, allocation, metadata, indirection,
checks, atomics, fences, or machine events absent from the reference design.

This bounded pass answers only:

1. whether Candidate C deserves to remain the first validation hypothesis;
2. which exact gaps belong to a family, family composition, C0 machine leaves,
   or compiler/optimizer quality; and
3. whether the evidence supports `C-SURVIVES`, `C-REVISE`, or `C-FAILS`.

It does not select a production design or establish safety, derivability, code
shape, or measured-performance closure.

## 2. Candidate roles

- Candidate C is the first bounded validation hypothesis.
- Candidate B is retained as the compression challenge: after necessary C
  families are evidenced, B asks which can be merged into a smaller common
  algebra without losing exact behavior or adding cost.
- Candidate A is retained as the generality fallback if finite families and
  bounded composition cannot express a required efficient mechanism.

These are research roles, not production rankings.

## 3. Evidence states

Every audited operation receives exactly one state:

- `ROUTED`: one frozen C family directly expresses the exact operation.
- `COMPOSED`: frozen families compose without an extra structural event.
- `FAMILY-GAP`: an irreducible low-level state machine is absent.
- `COMPOSITION-GAP`: the needed families exist but their checked composition
  rule is absent or ambiguous.
- `C0-GAP`: an exact allocation, atomic, external, target, or image-lifecycle
  machine leaf is absent.
- `TAXED`: the route exists only with an extra initialization, zeroing, copy,
  move, allocation, metadata item, indirection, branch/check, atomic/fence,
  machine event, scan, or asymptotic cost.
- `OPTIMIZER-GAP`: semantics suffice but required code shape is not established.
- `UNKNOWN`: the bounded evidence slice cannot decide.

`UNKNOWN` is unresolved, never a pass. Paper routing is not formal safety,
implementation, code-shape evidence, or performance measurement.

## 4. Audit record

Each operation row records:

1. pinned project, revision, source identity, and operation;
2. observable contract and frozen reference representation;
3. required live-state, owner traffic, provenance, failure, cleanup, and fact
   behavior;
4. Candidate C family route and any C0 dependency;
5. reference structural events and every forced delta;
6. evidence state; and
7. one observable falsifier or unresolved question.

The mandatory cost dimensions are initialization/zeroing, payload copy/move,
allocation, metadata, indirection, branch/check, scan, atomic/fence, machine
event, code size, and asymptotic behavior.

## 5. Scope controls

- Reuse the existing Rust 1.97.0 census, 276 coverage clusters, 49 operational
  obligations, dense ledger, and witnesses. Do not rebuild their census.
- Freeze Candidate C before each audit. Do not revise it while scoring the same
  evidence.
- Freeze one subsystem and at most five operations before source inspection.
- Stop when the time box expires. Record `UNKNOWN`; do not widen the subsystem.
- A new family may be proposed only after at least two independent demands need
  the same state machine, existing composition has a concrete cost or safety
  failure, the rule is finite and deterministic, it contains no project or
  container identity, and unrelated programs pay no cost.
- Separate in-memory families, C0 machine leaves, optimizer quality, and
  algorithmic policy. A missing file or atomic event is not a container family.
- Every completed stage gets one decision-gates entry, required MCTS-Mem
  maintenance, verification, and a separate commit.

## 6. Explicit non-goals

This plan does not authorize:

- a language, syntax, specification, checker, compiler, backend, runtime,
  standard-library, container, or production-fact change;
- candidate construction or execution;
- a safety model, prototype, benchmark, scored experiment, AI trial, E0.1
  restart, or xlc migration;
- implementation of Hashbrown, an allocator, SQLite, Crossbeam, Tokio,
  Wasmtime, or any other audited project;
- a whole-project unsafe census or an audit beyond the frozen operations; or
- calling structural parity measured performance or exact D-2/P-1 closure.

## 7. Stage 0 — freeze Candidate C v0 and the rubric

Time box: 30 minutes. Stop rather than expand scope.

Produce `CANDIDATE-C-V0-AUDIT-BASELINE.md` containing:

1. the frozen C0 and C-1 through C-12 family inventory;
2. each family's exact boundary and representation-charged runtime state;
3. allowed cross-family compositions and forbidden authority inference;
4. the family-admission rule;
5. the evidence and cost taxonomy from this plan;
6. the audit-row schema;
7. universal safety/no-tax invariants; and
8. Stage 0 stop conditions and unresolved definitions.

Stage 0 fails closed if the existing candidate cannot be made into an
unambiguous audit baseline without adding a capability. On failure, stop before
Stage 1 and return the missing definitions to the owner.

## 8. Stage 1 — bounded Hashbrown calibration

Time box: 60 minutes. The time box covers evidence inspection and analysis;
mechanical verification and durability work do not enlarge the audit scope.

Before inspection, pin one official Hashbrown revision and exact source
identities. Audit exactly five operations:

1. lookup;
2. vacant insertion;
3. replacement;
4. removal; and
5. rehash.

Do not audit all traits, allocators, serialization, parallel features,
compatibility surfaces, or the whole repository.

Produce:

- `CANDIDATE-C-HASHBROWN-AUDIT.tsv`, with no more than 25 operation rows;
- `CANDIDATE-C-HASHBROWN-AUDIT.md`, containing the source pin, scope, exact
  routes, structural account, safety obligations, unresolved questions, and
  Gate 1 decision; and
- a deterministic verifier for scope, state vocabulary, row count, required
  fields, exact five-operation coverage, and fail-closed conclusions.

The audit must answer:

- how control metadata authorizes the corresponding payload place;
- how group/SIMD search results become version-bounded occupancy facts;
- how control and payload versions remain synchronized;
- exact owner disposition for insertion, duplicate/replacement, removal,
  failure, and rehash;
- whether rehash relocates each live payload exactly once;
- entry/result provenance and invalidation;
- exact live-place destruction; and
- whether unrelated dense/fixed shapes acquire sparse-family cost.

## 9. Gate 1

Stop after the Stage 1 report. Do not enter Stage 2.

Recommend later work only if all five operations avoid project-name
recognition, hidden initialization/zeroing/copy, whole-table scans outside the
reference algorithm, unrelated-shape tax, and unresolved provenance or cleanup
authority. `UNKNOWN`, `FAMILY-GAP`, `COMPOSITION-GAP`, `C0-GAP`, `TAXED`, or
`OPTIMIZER-GAP` remains explicit and prevents a pass claim.

The Gate 1 report chooses exactly one research disposition:

- `C-SURVIVES`: the frozen families route all five operations without a known
  structural tax; this authorizes nothing further.
- `C-REVISE`: a finite reusable family, composition, or C0 repair is needed;
  list it but do not modify Candidate C without owner authorization.
- `C-FAILS`: the slice requires project-specific recognition, open-ended proof
  authority, unbounded family growth, or unavoidable structural tax.

## 10. Later stages — not authorized

Stage 2 would be a separately authorized bounded allocator slice. Later
independent slices may cover Crossbeam reclamation/work stealing, SQLite B-tree
and persistence, Tokio suspension/cancellation, and Wasmtime executable-image
lifecycle. Each later slice must freeze one subsystem, at most five operations,
and its own time box. No later stage starts from this plan alone.

## 11. Required final handoff for the current authorization

The current work ends with:

1. this controlling plan committed;
2. Stage 0 baseline committed separately;
3. Stage 1 matrix, report, and verifier committed separately;
4. both repository gates green after the work;
5. owner changes preserved; and
6. a plain-language Gate 1 report that requests, but does not assume, any next
   authorization.
