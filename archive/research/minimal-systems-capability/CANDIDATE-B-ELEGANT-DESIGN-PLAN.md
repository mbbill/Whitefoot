# Candidate B Elegant Design Plan

Date: 2026-07-15

Status: controlling bounded paper-design and source-audit contract. The owner
authorized using the same finite comparative method applied to Candidate C to
determine whether Candidate B can become the more elegant primary research
hypothesis. Work stops at the Candidate B Design Gate in this file.

## 1. Question and outcome

The question is not whether one more language form can encode Hashbrown. It is:

> Can a small, closed, project-independent algebra let ordinary checked
> libraries express the native representations and owner traffic required by
> materially different high-performance systems, without growing one language
> family per container and without accepting arbitrary writer proofs?

The outcome must be understandable without reading the audit internals. It
will name exactly three Candidate B architectures and, for each one, explain:

1. what the capability set contains;
2. why every component is present and what fails if it is removed;
3. which concrete performance blocker each composition removes;
4. how an ordinary library derives every audited operation;
5. what safety, cleanup, provenance, concurrency, and implementation questions
   remain;
6. the advantages and disadvantages relative to the other two architectures;
   and
7. one fail-closed gate disposition.

"Elegant" is not a style score. A candidate is more elegant only if fewer
independent semantic rules cover more independent projects and operations,
extension pressure is lower, runtime cost remains representation-selected, and
safety and implementation boundaries remain explicit.

## 2. Frozen evidence set

### 2.1 Reused evidence

Reuse the completed Hashbrown audit without reopening its source scope:

- Hashbrown v0.17.1, commit
  `c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d`;
- the five audited operations and their exact source references;
- Candidate C's sparse repair comparison; and
- the existing Candidate A/B/C definitions, demand ledger, hostile review, and
  MCTS-Mem design record.

### 2.2 New primary-source slices

Inspect only these official pinned revisions:

- mimalloc v3.3.2, annotated Git tag object
  `5687270e7fbb15d494a46b0d048f978bad973e4f`, dereferenced source commit
  `30b2d9d89099bee08e9f67a1ffb3e12e7ba45227`;
- SQLite 3.53.3, official Fossil source ID
  `d4c0e51e4aeb96955b99185ab9cde75c339e2c29c3f3f12428d364a10d782c62`
  and official Git mirror tag commit
  `92a6c5c3636faa021ecc3be5403a00f50f65eda7`; and
- Crossbeam Epoch 0.9.18, monorepo tag commit
  `9c3182abebb36bdc9446d75d4644190fef70fa01`.

The Git identity is a retrieval pin. For SQLite, the Fossil source ID is the
canonical identity.

These projects are not evidence that Candidate B is fast. They expose
representation, ownership, cleanup, provenance, and concurrency requirements
that a no-tax language route must preserve.

## 3. Frozen operations

Audit exactly fourteen operations. Do not add a fifteenth operation in response
to an interesting source path.

### Hashbrown: sparse storage and partial relocation

1. `H-LOOKUP`: lookup through control metadata into an initialized payload.
2. `H-INSERT`: vacant insertion and offered-owner commit.
3. `H-REPLACE`: duplicate replacement and displaced-owner return.
4. `H-REMOVE`: removal and returned-owner accounting.
5. `H-REHASH`: resize or in-place rehash with partial progress.

### mimalloc: allocator-local and cross-thread lifecycle

6. `M-ALLOC`: small-block allocation from a page-local free structure.
7. `M-LOCAL-FREE`: same-owner-thread free and page state update.
8. `M-REMOTE-FREE`: cross-thread delayed free, collection, and page or segment
   lifecycle transition.

### SQLite: recursive pages and transactional failure

9. `S-INSERT-SPLIT`: B-tree insertion including a page split or rebalance.
10. `S-DELETE-BALANCE`: B-tree deletion including cell removal, page
    rebalance, and page release.
11. `S-ROLLBACK`: rollback of partially completed page changes through the
    pager transaction machinery.

### Crossbeam Epoch: shared access and deferred reclamation

12. `X-PROTECTED-LOAD`: pinning and loading a shared pointer under a guard.
13. `X-RETIRE`: unlinking or retiring an owner and deferring destruction.
14. `X-COLLECT`: epoch advancement, quiescence determination, and execution of
    eligible deferred destruction.

Together these operations test sparse liveness, direct owner transitions,
partial relocation, allocator metadata and cross-thread ownership, recursive
page topology, transactional rollback, guarded shared access, callback cleanup,
quiescence, and policy-defined reclamation. They do not establish ecosystem
completeness.

## 4. Frozen design alternatives

Derive exactly three alternatives before routing any operation:

- `B-FORMS`: the original flat closed-form design. Full, Prefix, Ring, Sparse,
  Product, Hole, fixed cleanup, footprints, handles, and shared-lifecycle forms
  remain individually compiler-known.
- `B-STRATA`: the proposed compression design. It separates physical places,
  structural live-set descriptions, primitive owner transitions, scoped
  protocol state, physical-root provenance, executable cleanup, invalidatable
  facts, and concurrent access safety. Ordinary libraries compose a closed
  grammar; compiler specialization changes code shape only.
- `B-GRAPHS`: ordinary libraries define bounded protocol graphs over a small
  primitive vocabulary. This is the extensibility control and ceases to be B if
  admission needs arbitrary invariants, theorem proving, unbounded graphs, or
  writer-defined cleanup authority.

Do not add, merge, rename, or revise an alternative after routing begins.

## 5. Source-audit method

For every operation, record:

1. exact project, revision, file, symbol, and line or commit-stable source
   anchor;
2. the physical representation and which storage may be uninitialized;
3. the metadata or state that authorizes payload access;
4. exact owner inputs, outputs, moves, returns, and destruction;
5. normal, recoverable-failure, abandonment, and partial-progress behavior;
6. cleanup traversal, recursion, callback, and termination requirements;
7. physical allocation roots and derived-reference provenance;
8. shared access, publication, interference, atomic ordering, quiescence, and
   reclamation policy where present;
9. structural operations that the source performs; and
10. every extra initialization, zeroing, copy, relocation, owner movement,
    metadata field, indirection, check, allocation, synchronization event, scan,
    code-shape requirement, or asymptotic change that would violate the route.

Unknown source behavior stays `UNKNOWN`. Unsafe source code reveals an
obligation; it does not transfer authority into xlang.

## 6. Required definition for each alternative

Each alternative must begin in plain language and then define:

1. a complete capability inventory;
2. the inclusion reason and removal witness for every capability;
3. what ordinary library code may define and compose;
4. what remains a closed compiler or checker rule;
5. physical storage, liveness, and access admission;
6. primitive owner transitions and logical commit;
7. non-escapable multi-step protocol state and rollback;
8. physical-root provenance across wrapper, token, page, or registry moves;
9. executable cleanup, termination, recursion, callbacks, and partial progress;
10. optimizer facts and exact invalidation;
11. shared-access safety separated from synchronization and reclamation policy;
12. checker, compiler, backend, runtime, diagnostic, and AI-writer shape;
13. all eleven structural-cost dimensions;
14. the project-independent extension rule and measured extension pressure;
15. convergence risk toward C or A;
16. unresolved questions; and
17. observable falsifiers.

Paper notation may describe rules but is neither syntax nor implementation.

## 7. Route states and comparison matrix

Produce exactly 42 candidate-operation rows: three alternatives times fourteen
operations. Every row uses one state:

- `CLOSED`: finite paper rules cover the operation with no identified
  structural event beyond the pinned source contract.
- `OPEN`: a named semantic, safety, cleanup, provenance, concurrency, or policy
  rule remains undefined.
- `TAXED`: the route adds a runtime event, field, tag, owner movement, scan,
  allocation, synchronization operation, code-size requirement, or asymptotic
  cost absent from the source contract.
- `CONVERGES-C`: closure requires a topology-, project-, container-, operation-,
  or policy-specific family or compiler semantic path.
- `CONVERGES-A`: closure requires arbitrary writer propositions, invariants,
  proofs, or theorem checking.
- `UNKNOWN`: the bounded evidence cannot decide.

Only `CLOSED` supports selection. Every row records the composed capabilities,
performance blocker removed, owner and cleanup account, provenance, concurrency
account, runtime state, structural delta, extension effect, unresolved issue,
and one falsifier.

The report compares at least conceptual rule count, independent projects and
operations per rule, representation freedom, cleanup generality, provenance
precision, concurrency-policy freedom, checker state, compiler and backend
specialization, runtime metadata, code-size risk, diagnostic locality,
writer-visible complexity, AI stability, extension pressure, interaction
growth, and C/A convergence.

## 8. Binding design constraints

1. Project, container, algorithm, API, path, and policy identities remain
   ordinary library concepts.
2. Owning raw places grants no payload read, move, borrow, or destruction.
3. Live-set descriptions grant access only to exact places under one physical
   root relation and version.
4. Every owner is retained, moved, returned, or destroyed exactly once on each
   normal or recoverable outcome.
5. An intermediate protocol state cannot escape or be interpreted by an
   operation that expects a public stable state.
6. A liveness description alone cannot execute cleanup. Cleanup needs an
   executable, verified, terminating disposition mechanism.
7. Recursive cleanup may descend only by consuming a strict owned child of an
   acyclic affine owner; shared cycles require a separately selected policy.
8. Callback contracts state call cardinality, effects, result provenance, owner
   disposition, and exceptional behavior; callback identity grants no payload
   authority.
9. Borrow provenance is rooted in physical allocation identity, never in a
   movable registry slot, wrapper, guard field, page number, or integer handle.
10. Shared-access rules state safety facts, not a synchronization or
    reclamation product policy.
11. Compiler recognition may specialize a verified composition but may not
    make an otherwise invalid program valid.
12. Unused capabilities add no runtime field, tag, branch, load, call,
    allocation, atomic, fence, scan, machine event, or required code path.

Any violation disqualifies a route rather than becoming a weighted cost.

## 9. Extension discipline

A new B rule may be proposed only when:

1. at least two independent projects need the same irreducible relation;
2. every composition of current rules has a named safety failure or structural
   tax;
3. the rule is finite, deterministic, locally checkable, and contains no
   project, container, algorithm, API, path, or policy identity;
4. it defines exact normal, failure, abandonment, cleanup, provenance, fact,
   and invalidation behavior where applicable;
5. unrelated programs pay zero runtime and required-code cost; and
6. the proposal states how many old rules or special cases it replaces.

A rule that serves one operation or one project only is an unresolved
abstraction, not an automatic extension.

## 10. Phases, limits, and stop rules

### Phase 0: freeze and durability

Pin the four inputs, fourteen operations, three alternatives, route vocabulary,
matrix schema, and gate in this separate plan commit.

### Phase 1: source demands

Reuse Hashbrown and inspect the three new projects. Each new project receives
at most thirty analyst-minutes and exactly three operations. The three slices
may run in parallel. Definition chasing is allowed only when necessary to
account for one frozen operation. Stop unclear paths as `UNKNOWN`.

### Phase 2: architecture and routing

Derive the three alternatives before scoring. Architecture derivation and all
42 routes receive at most sixty analyst-minutes. No candidate may be repaired
after seeing its score.

### Phase 3: comparison and gate

Use at most thirty analyst-minutes for the pros/cons comparison, hostile
self-review, verification, and gate. Stop after the gate.

This plan does not authorize:

- modifying the frozen Candidate A, B, or C definitions;
- language, syntax, specification, checker, compiler, backend, runtime,
  standard-library, container, or pattern-doctrine changes;
- whole-project source surveys, unsafe censuses, additional projects, or a
  fifteenth operation;
- a formal safety model, mechanized proof, prototype, candidate construction,
  execution, generated-code inspection, benchmark, measurement, or AI trial;
- implementing mimalloc, SQLite, Crossbeam, Hashbrown, or any reclamation
  policy in xlang;
- Stage 2 of the Candidate C plan; or
- selecting a production design.

## 11. Deliverables

Produce and commit:

1. this plan;
2. `CANDIDATE-B-MULTIPROJECT-AUDIT.md`, a readable account of the fourteen
   concrete operations;
3. `CANDIDATE-B-MULTIPROJECT-AUDIT.tsv`, containing exactly fourteen operation
   rows, plus a deterministic source-audit verifier;
4. `CANDIDATE-B-ELEGANT-DESIGN.md`, written first for owner comprehension and
   then for exact verification;
5. `CANDIDATE-B-ELEGANT-DESIGN-MATRIX.tsv`, containing exactly 42 rows; and
6. a deterministic design verifier for identities, row coverage, route
   vocabulary, required fields, gate consistency, and fail-closed status.

## 12. Candidate B Design Gate

Stop after the deliverables and choose exactly one disposition:

- `B-SELECT`: one alternative closes all fourteen operations, violates no
  binding constraint, adds no identified structural tax, and remains distinct
  from C and A. Name it only as a further-research hypothesis.
- `B-REVISE`: B remains plausible but every alternative has at least one open,
  taxed, convergence, or unknown row.
- `B-NONE`: the bounded evidence shows that B cannot close without repeated
  special families, general proof authority, or structural tax.

The gate authorizes no implementation, safety model, further audit, or
production decision. Any next step requires separate owner authorization.
