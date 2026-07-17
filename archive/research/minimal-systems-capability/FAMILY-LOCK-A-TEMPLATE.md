# Minimal Systems Capability Basis: Family Lock A Template

Status: uninstantiated G0-Core research template, 2026-07-14. This file is not
a Family Lock A and selects no family, caller contract, candidate mechanism,
algorithm, representation, workload, payload, target, allocator, numeric
margin, fact channel, language change, or implementation. Instantiating it
requires a separate owner authorization after G0-Core closes. A completed lock
authorizes only the activities stated in its owner authorization record.

This template carries forward the boundaries in `G0-CORE-CHARTER.md`,
`SEMANTIC-OBLIGATION-REGISTRY.md`, `CAPABILITY-OBLIGATION-REGISTRY.tsv`,
`RUST-DATA-CONTRACT-CENSUS.tsv`, `WITNESS-REGISTRY.md`,
`SYSTEMS-DOMAIN-LEDGER.md`, and `E01-TRACEABILITY.md`. It is mechanism-neutral:
it requires exact candidate and reference mechanisms to be named by a future
lock, but supplies none.

## 1. Instantiation rules

Every field marker has the form `<required: ...>`. Before lock review, replace
every marker with an exact value or with an explicit `NOT-APPLICABLE` record
that cites the frozen contract and proves why the field cannot affect
soundness, performance, selection, or scope. Deleting a marker, using a generic
catch-all, or leaving a prose ambiguity means the lock is incomplete.

An instantiated lock must satisfy all of the following:

1. It freezes one family and all cross-family semantics exposed by any
   candidate in that lock.
2. It contains no candidate implementation or scored result.
3. It gives every candidate the same visible semantic contract, construction
   policy, correction policy, and scoring information unless an asymmetry is
   preregistered and justified.
4. It hashes every exact input that can affect implementation, acceptance,
   scoring, or review.
5. It resolves every applicable global law G-1 through G-16 and every
   implicated capability, witness, deferral, and historical E0.1 input.
6. It receives independent hostile review on exact bytes before candidate
   construction begins.

## 2. Lock identity and authorization

| Field | Required locked value |
|---|---|
| Lock identifier | `<required: stable family-lock identifier>` |
| Family name | `<required: exactly one family>` |
| Lock revision | `<required: immutable revision identifier>` |
| Lock date and timezone | `<required: exact timestamp and timezone>` |
| Lock author and reviewers | `<required: identities and review roles>` |
| Owner authorization to draft this lock | `<required: exact directive or decision-gates reference>` |
| Owner authorization granted by approving this lock | `<required: explicit allowed actions; candidate construction is not implied>` |
| Caller-visible family claim | `<required: one bounded claim sentence>` |
| Explicitly excluded claims | `<required: payload, ownership, lifetime, failure, concurrency, address, platform, and library-surface exclusions>` |
| Required predecessor locks | `<required: exact lock IDs and hashes, or NOT-APPLICABLE with dependency proof>` |
| Cross-family state/fact exposure | `<required: every public transition, state topology, trusted fact, or trusted path shared with another family>` |
| Current G0-Core status | `<required: closing commit and decision-gates entry proving G0-Core is complete>` |

The lock is invalid if it depends on an unlocked family or if a supposedly
family-local candidate exposes semantics whose admissible behavior spans an
unfrozen family.

Every lock freezes a required predecessor DAG for imported family contracts and
witness execution. A witness importing `FAM-X` cannot be an adoption or closure
gate for that same imported family; it must name `FAM-X` as a closed predecessor
and run only in a post-adoption witness stage. The import grants the predecessor's
public API and counters, never its representation, private facts, or unchecked
capabilities. All imported allocation, memory, traffic, and code-size counters
remain separately charged. Using a capability selected by another family
creates a predecessor edge even when no `FAM-X` token is imported. A held-out
assigned to the current family may exercise capabilities being selected in that
lock, but it may not import that family's completed `FAM-X` container. A dependency
cycle invalidates the lock.

## 3. Frozen input manifest

The instantiated lock must hash the exact bytes it relies on. A repository path
without a digest is not a frozen input.

| Input class | Required path or identity | Required digest or version | Locked use |
|---|---|---|---|
| G0-Core charter | `G0-CORE-CHARTER.md` | `<required: SHA-256>` | Research, derivation, performance, and authorization boundary |
| G0-Core synthesis | `G0-CORE-REPORT.md` | `<required: SHA-256>` | Closed research result, bounded next request, and claim language |
| G0-Core exact-artifact manifest | `G0-CORE-ARTIFACT-MANIFEST.json` | `<required: SHA-256>` | Reviewed research inputs and controlling-law/design-memory byte identities |
| Mechanical census manifest | `RUST-1.97.0-CENSUS-MANIFEST.json` | `<required: SHA-256>` | Exact Rust source, tool, inventory, module-ledger, count, and digest authority |
| Semantic obligations | `SEMANTIC-OBLIGATION-REGISTRY.md` | `<required: SHA-256>` | Global laws, proof criteria, fact schema, and deferrals |
| Capability obligations | `CAPABILITY-OBLIGATION-REGISTRY.tsv` | `<required: SHA-256>` | Exact implicated capability IDs and no-tax laws |
| Rust coverage-cluster census | `RUST-DATA-CONTRACT-CENSUS.tsv` | `<required: SHA-256>` | Coarse caller-demand and obligation envelopes; explicitly not exact operation contracts or import units |
| G0 coverage-cluster registry | `G0-COVERAGE-CLUSTER-REGISTRY.tsv` | `<required: SHA-256>` | Exact cluster bytes, non-importable semantic class, and Family Lock expansion policy |
| Rust surface map | `RUST-DATA-SURFACE-MAP.tsv` | `<required: SHA-256>` | Exact stable-safe declaration evidence identities and their primary G0 coverage clusters; stable-unsafe coverage is verified through evidence clusters |
| Rust D10 route map | `RUST-D10-SURFACE-MAP.tsv` | `<required: SHA-256>` | Exact iteration/range declaration routes, including redundant-surface identities |
| Stable-unsafe evidence map | `RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv` | `<required: SHA-256>` | Exact evidence-only routing for every stable-unsafe data-floor declaration |
| Concrete trait-implementation crosswalk | `RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv` | `<required: SHA-256>` | Exact concrete implementation and stable-reachability evidence identities |
| Concrete trait-implementation topology routing | `G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv` | `<required: SHA-256>` | Exact primary refinement family/gate, predecessor, and implicated/reopening targets for every concrete implementation key; no fuzzy or default classifier |
| Coverage evidence universe | `G0-COVERAGE-EVIDENCE-UNIVERSE.tsv` | `<required: SHA-256>` | Machine-derived complete evidence-key relation for every G0 cluster |
| Cluster family routing | `G0-CLUSTER-FAMILY-ROUTING.tsv` | `<required: SHA-256>` | Exact coarse-cluster primary refinement stage, predecessors, implicated/reopening families, and route state; never a closure unit |
| Family role and canary registry | `G0-FAMILY-REQUIREMENT-REGISTRY.tsv` | `<required: SHA-256>` | Exact Section-4 plus witness-registry B/M/W/H/O obligation union and family policies |
| Family and gate vocabulary | `G0-FAMILY-GATE-VOCABULARY.md` | `<required: SHA-256>` | Human-readable identities, applicability equations, assignments, rationales, and source authority |
| D11 capability-matrix source | `../general-purpose-data-structure-capability-RESEARCH.md` | `<required: SHA-256>` | Owner-ratified role, staging, family, and closure boundary |
| Contract derivation matrix | `DERIVATION-MATRIX.tsv` | `<required: SHA-256>` | Coarse cluster-to-capability screens, gaps, costs, facts, canaries, and later gates; not exact member derivations |
| Payload-scope classification | `PAYLOAD-SCOPE-CLASSIFICATION.tsv` | `<required: SHA-256>` | Exact six-state per-cluster stored-borrow partition and machine-readable `scope_owner_contract_ids` |
| Payload-scope overlay | `PAYLOAD-SCOPE-OVERLAY.tsv` | `<required: SHA-256>` | Cluster-scoped conditional payload-role obligations, capability deltas, dispositions, and reopening triggers that must be rebound to exact lock-local units |
| Witness registry | `WITNESS-REGISTRY.md` | `<required: SHA-256>` | B, W, O, and H contracts, dependency budgets, and custody |
| Systems-domain ledger | `SYSTEMS-DOMAIN-LEDGER.md` | `<required: SHA-256>` | Detailed-floor boundary and deferred whole-envelope claims |
| E0.1 traceability | `E01-TRACEABILITY.md` | `<required: SHA-256>` | Historical arm, control, and measurement disposition |
| Current specification | `<required: exact spec path>` | `<required: SHA-256 and rule-set version>` | Current semantics and proposed delta base |
| Derivation ledger | `<required: exact ledger path>` | `<required: SHA-256>` | META-5 base |
| Design tree | `<required: exact relevant nodes and .alt paths>` | `<required: SHA-256 per file>` | In-force decisions and defeated alternatives |
| Pattern doctrine | `<required: exact pattern artifact>` | `<required: SHA-256>` | COMPLETE and EFFICIENT acceptance boundary |
| Current plan and owner directives | `<required: exact paths>` | `<required: SHA-256 per file>` | Staging and authorization boundary |
| Rust source anchor | `<required: release, tag object, peeled commit, and tree>` | `<required: exact values>` | External contract and same-shape source authority |
| Toolchain baseline | `<required: compiler, linker, runtime, target, and tool versions>` | `<required: executable and configuration digests>` | Reproducible construction and measurement |
| Repository baseline | `<required: clean commit and permitted uncommitted state>` | `<required: commit ID and status digest>` | Candidate starting point |

Any later input-byte change follows Section 20 rather than being silently
accepted as a refresh.

## 4. Family boundary and dependency closure

### 4.1 Coverage-cluster and evidence-key accounting

List every G0 coverage cluster assigned to or implicated by the family and its
complete evidence-key universe. For each cluster, that universe is the union of
all matching stable-safe keys in `RUST-DATA-SURFACE-MAP.tsv`, all routed keys
in `RUST-D10-SURFACE-MAP.tsv`, stable-unsafe keys in
`RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv`, concrete implementation keys in
`RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv`, and explicit helper or protocol
selectors named by the cluster row. A filtered subset is not a completeness
universe.
The 276 G0 rows are coarse obligation envelopes with semantic class
`G0_COVERAGE_CLUSTER`; they are not exact operation contracts, closure units,
experiment units, or Family Lock imports. Their capability and cost fields may
be conservative unions and therefore cannot be copied into an exact member row.
`G0-COVERAGE-CLUSTER-REGISTRY.tsv` binds that class and policy to the exact
census and matrix row bytes.

Before candidate construction, the instantiated lock must provide a
machine-readable evidence-disposition ledger, role-disposition ledger,
member/outcome ledger, canary ledger, and a fail-closed validator with exact
source and executable hashes. The validator derives expected sets from the
frozen G0 inputs rather than from rows selected by the lock. It rejects missing,
duplicate, unknown, or illegally excluded rows and proves all of the following:

1. the current-family primary and implicated/reopening cluster sets equal the
   applicable rows in `G0-CLUSTER-FAMILY-ROUTING.tsv` for the frozen family or
   gate-stage ID and every exposed cross-family state, transition, fact, or
   trusted path; separately referenced predecessor proof units equal the exact
   required predecessor sets, but predecessor membership never makes a
   downstream cluster applicable backwards;
2. the evidence-key set equals the complete audit-domain relation in
   `G0-COVERAGE-EVIDENCE-UNIVERSE.tsv` for those clusters; for each evidence
   identity `e`, the validator independently derives its exact applicable target
   set `A(e)`, and the current family or gate has exactly one terminal
   disposition if and only if it is in `A(e)`, plus one terminal for every
   additional applicable gate; no target in `A(e)` is orphaned globally;
3. every applicable row in `G0-FAMILY-REQUIREMENT-REGISTRY.tsv` has exactly one
   legal owner-lock disposition; each cross-topology implicated-family use has
   its separate compound reuse-and-local-rebind disposition; every B control
   runs against every candidate; M/W/H non-inclusion blocks the assigned family
   and complete-floor claim; and `OPTIONAL_NOT_PROMOTED` is used only for an O
   row that no mandatory contract requires;
4. every required primary and cross-cut canary maps to exact member/outcome
   units and frozen executable or generated-fixture bytes;
5. every refined declaration, implementation, helper, protocol, capability,
   payload branch, witness, and canary maps to exact `member_contract_id` and
   `outcome_id` units without inheriting a cluster-wide union; and
6. member/outcome totality covers every normal, recoverable-failure,
   checked-trap, abandonment, and destruction outcome admitted by the exact
   contract.

For a lock owned by family `F`, the applicable role set is exactly: every B
row; every non-O row whose `closure_owner_or_gate_stage` is `F`; and every O row
explicitly promoted by the owner or required by a mandatory contract in that
lock. A post-adoption row belongs only to its exact named gate stage. The
registry's `implicated_family_ids` affect claim boundaries and reopening, not
closure ownership. The lock's predecessor set must equal the union of
`required_predecessor_family_ids` for its applicable rows, after exact
predecessor proofs are resolved; it may neither omit a required predecessor nor
turn an implicated family into an imported predecessor.

A cluster route places that cluster's complete evidence universe in the current
lock's audit domain; it does not make every child applicable to the lock. For
each exact identity `e`, `A(e)` comes from an independent child-applicability
authority. A concrete trait-implementation identity uses its exact `impl_key`
topology route composed with the closed owning-cluster operation-gate assignment
in `G0-FAMILY-GATE-VOCABULARY.md`. Its exact target set is the topology primary
plus the additional operation gate when that cluster has one; topology
predecessors are dependencies rather than members of `A(e)`. A direct key or
selector child requires a lock-local exhaustive applicability ledger before
disposition. If the current family or gate is not in
`A(e)`, the lock records the routed non-applicable child but may not refine it,
claim predecessor proof for it, or exclude it. One target's exclusion never
erases another target's obligation. The global validator proves that every
target in every `A(e)` receives exactly one legal terminal. Cluster routing
never transfers the cluster's `E`/`P` status, capability union, cost envelope,
or closure to a family.

Cross-cut dimensions such as access/reborrow, ownership transition, failure,
and behavior are not predecessor families. They appear under exact `DIM-*`
identities in the routing registry and must be bound locally to the applicable
member/outcome units of the current family. A lock may not invent an unclosed
`F-*` predecessor from ordinary `BR-*`, `OW-*`, `EX-*`, `FL-*`, or `AB-*`
capability obligations. Genuine scoped later families remain explicit `F-*`
routes with claim-blocking exclusions and reopening triggers.

For a route marked as cross-topology or per-evidence-child rebind, the lock must
also supply an independently derived evidence-child applicability ledger. Its
key set equals every exact evidence identity and every independently expanded
selector child for the cluster. Each row binds implementer, declaration path,
or source identity to all applicable family/gate IDs and the exact derivation
rule or source-audit record. Cross-cut trait clusters therefore expose their
dense, deque, linked, sparse, ordered, heap, text, shared, dynamic-borrow, and
other concrete implementation children to the corresponding locks. A
self-selected child-to-family map, a cluster-primary shortcut, or exclusion
without this independently reviewed applicability evidence is invalid.
For every concrete implementation identity, the validator additionally joins
the exact `impl_key` to `G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv` and proves equality
of its primary refinement family or gate, predecessor set, and
implicated/reopening set. It also joins the owning cluster to the closed
operation-gate assignment and proves the additional gate and its immediate
child-specific predecessor. When present, that predecessor is exactly the
topology primary; it is additive to, not a replacement for, the operation gate's
complete route-level predecessor family/gate set. The two targets are distinct
and require separate terminals, while the topology primary retains its own
predecessor sets. An ungated concrete identity carries
an explicit closed `NONE`/`NONE`, not missing evidence. A cluster-level family
union cannot override that child route. Exclusion in one family does not erase
the same child's independently applicable local-rebind obligation elsewhere.

A row marked `EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY` is also an
accounting obligation in every listed implicated family. Its closure owner
freezes the reusable contract, but each implicated lock must bind that family's
topology-specific traversal members, outcomes, invalidation, costs, and
canaries. The only successful implicated-family role terminal is the compound
`PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED`; predecessor reuse or a local
topology record alone cannot discharge a deque, sparse, ordered, heap,
identity, recursive, arena, text, or other topology-specific traversal.

No candidate construction, Candidate Freeze B, `E` or `P` claim, scored or
held-out execution, family closure, or adoption request may begin until that
validator passes, independent hostile review passes on the same exact lock and
validator bytes, and the owner separately authorizes the next action.

| G0 coverage cluster ID | Applicable G0 obligation IDs and roles | Complete evidence-key audit domain | Exact target sets and per-applicable-target terminals | Obligation-envelope summary | Candidate member-contract IDs | B/W/H dependencies | Claims blocked by exclusions |
|---|---|---|---|---|---|---|---|
| `<required: exact G0 cluster ID>` | `<required: complete many-to-many obligation IDs with each frozen role>` | `<required: every safe, D10, unsafe, concrete-impl, helper, and protocol key>` | `<required: independently derived A(e); one terminal per applicable (e, target), none for non-applicable targets>` | `<required: bounded coarse demand; do not claim member equality>` | `<required: complete stable member_contract_id list>` | `<required: exact witness IDs>` | `<required: exact family, cluster, and whole-floor claims or NONE>` |

The final table must record every census row assigned to the family, every key in
each routed cluster's complete audit domain, and every neighboring row whose
implementation would consume a proposed public transition, state topology,
fact, or trusted path. There is no ignored or missing audit input. Each key has
an independently derived target set, and there is no ignored or not-selected
disposition for an applicable `(evidence_identity, target_id)` pair. A
non-applicable target has no terminal in this lock. Every `REFINED_IN_LOCK` key
for the current target maps to exactly one
`member_contract_id`; every `PREDECESSOR_PROVED` key names the exact frozen
predecessor unit; every `EXCLUDED_BLOCKS_CLAIM` key names each blocked family,
cluster, and whole-floor claim. Multiple keys may share one member contract only when the
lock proves exact equality of result, ownership, provenance, invalidation,
failure, drop, complexity, layout, identity, order, behavior-call, allocation,
and resource guarantees. Similar names, directionality, or a shared G0 cluster
are not that proof.

For each selector relation, the lock records the parent
`selected_source_value_sha256`, the exact materialized child count, the ordered
child-identity digest, every child's independently derived exact target set, and
one terminal disposition for every applicable child-target pair. The child
set must equal an independently derived exhaustive expansion of the frozen
`selected_source_value`: use a frozen expansion grammar and tool for parseable
selectors, or an independent source-audit ledger binding every token or anchor
to exact Rust source identities and hashes. A refined selector requires that
set equality plus every terminal applicable to the current target.
`PREDECESSOR_PROVED` requires the exact predecessor child universe, applicable
terminal records, lock/unit identities, and hashes.
`EXCLUDED_BLOCKS_CLAIM` blocks every claim that depends on any unresolved child.
Parent-only or partial expansion is invalid; a self-selected child count and
digest is not completeness evidence.

| Selector evidence identity | Parent selected-source-value SHA-256 | Independent expansion authority and SHA-256 | Exact child count | Ordered child-identity digest | Per-child target and terminal ledger | Disposition |
|---|---|---|---|---|---|---|
| `<required: exact selector identity>` | `<required: frozen parent value hash>` | `<required: parser/grammar or independent source-audit ledger plus exact hashes>` | `<required: exhaustive count>` | `<required: digest over independently derived ordered identities>` | `<required: A(e) for every child; one terminal per applicable target and none for a non-applicable target>` | `<required: refined, predecessor-proved, or claim-blocking exclusion for each applicable pair>` |

### 4.1.1 B/M/W/H/O role and canary dispositions

For every validator-derived applicable row in
`G0-FAMILY-REQUIREMENT-REGISTRY.tsv`, record exactly one owner-lock disposition.
An M, W, or H row may use only `REQUIRED_IN_LOCK` or
`EXCLUDED_BLOCKS_FAMILY_AND_FLOOR`; a predecessor is an input and cannot prove
the owner's requirement. A B row must use `PROTECTED_CONTROL`; a missing or
failing B row rejects candidate construction and scoring. An O row may also use
`OPTIONAL_NOT_PROMOTED`, but only when no mandatory contract or selected
candidate requires it.

Only `EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY` creates a role terminal in
an implicated lock. Success requires the indivisible
`PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED`, which names the closure owner's
exact reusable lock/member/outcome bytes and the current family's exact local
topology/member/outcome/canary bytes. Claim-blocking exclusion is the only
other mandatory terminal. All other implication policies use
`NOT_APPLICABLE_REOPENING_ONLY`: they affect claim boundaries and reopening but
do not create a second role-closure obligation.

The compound reuse record is valid only when every affecting specification,
compiler, lowering, optimizer, toolchain, target, shared-capability, and public
dependency byte or frozen identity is unchanged. The validator compares that
complete predecessor environment, not only the public contract and old result
hashes. Any mismatch either reruns every affected predecessor soundness,
structural, and performance canary against the current candidate/configuration,
or reopens the closure-owner lock; it cannot remain reusable by
assertion.

| G0 obligation ID | Role | Subject, owner/gate, predecessor, implicated-family, and rebind fields | Owner-lock disposition | Implicated-family rebind disposition | Exact proof, reuse authority, or blocked claims | Exact member/outcome units | Primary and cross-cut canary fixture IDs and hashes |
|---|---|---|---|---|---|---|---|
| `<required: exact obligation ID>` | `<required: B, M, W, H, or O>` | `<required: exact frozen IDs>` | `<required: legal owner-lock disposition or NOT_APPLICABLE_IMPLICATED_REBIND>` | `<required: compound reuse-and-local-rebind proof, claim-blocking exclusion, or NOT_APPLICABLE_REOPENING_ONLY>` | `<required: exact evidence and byte-identical environment>` | `<required: complete exact units or claim-blocking exclusion>` | `<required: exact fixture identities and SHA-256 values>` |

### 4.2 Capability applicability

For each row of `CAPABILITY-OBLIGATION-REGISTRY.tsv`, record one of:
`REQUIRED`, `PROTECTED`, `DEFERRED-BLOCKS-CLAIM`, or `NOT-IMPLICATED`, with a
member-and-outcome-backed reason. Wildcard or cluster-wide applicability is not
sufficient for closure.

| Capability ID | Applicability | Exact member_contract_id and outcome_id obligations | Candidate-visible dependency | Proof or fixture destination |
|---|---|---|---|---|
| `<required: capability ID>` | `<required: one allowed applicability state>` | `<required: exact family-local obligation or reason>` | `<required: public dependency or none>` | `<required: section, proof, or exact fixture ID>` |

At minimum, every owning family must explicitly dispose `OW-DROP`, `EX-NORMAL`,
`EX-ABORT`, `BR-INVALIDATE`, applicable `FL-*`, `AB-SEAL`, `AB-GENERIC`,
`FT-STATE`, `NT-FIXED`, and `NT-P2`. Listing them does not make them applicable
or solved; the row must say why.

### 4.3 Scoped deferrals

| Deferred domain | Included or excluded | Exact blocked claim | Candidate restriction | Reopening trigger |
|---|---|---|---|---|
| Borrow-bearing stored payloads | `<required: disposition plus applicable G0 overlay branch IDs rebound to exact member/outcome units>` | `<required: exact claim boundary>` | `<required: enforceable payload restriction and base-route scope>` | `<required: trigger>` |
| Shared ownership and weak identity | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Concurrency and atomic sharing | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Pinning and address-sensitive values | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| User-defined finalization | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Recoverable custom allocation | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Resources and FFI | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Async execution and cancellation | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Complete text and Unicode | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Target intrinsics and SIMD surface | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Dynamic dispatch and open-world behavior | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |
| Cyclic tracing or garbage collection | `<required: disposition>` | `<required: exact claim boundary>` | `<required: enforceable restriction>` | `<required: trigger>` |

An included row becomes a family dependency and must receive exact semantics,
candidate coverage, fixtures, performance accounting, and hostile review in
this or a predecessor lock. An excluded row may not be used by any candidate.
For stored-borrow scope, the lock must list every selected cluster's exact
classification, `scope_owner_contract_ids`, and applicable overlay branch keys,
then bind each selected key to exact lock-local member and outcome units.
It may not replace those keys with “all generic payloads,” infer adjacent
members, or promote a region-free/borrow-free experiment to unrestricted
contract closure. The frozen G0 input contains 26 active, 138 deferred, 100
true no-complement, nine boundary-evidence, two frame-scope-deferred, and one
delegated contract, plus 294 exact overlay branches.

| G0 cluster ID | Member contract ID | Outcome ID | Classification | Scope-owner cluster IDs | Overlay branch ID | Exact unit base capability IDs | Conditional delta | Exact effective capability IDs | Disposition and proof destination |
|---|---|---|---|---|---|---|---|---|---|
| `<required: exact G0 cluster ID>` | `<required: stable member_contract_id>` | `<required: stable outcome_id>` | `<required: exact six-state value>` | `<required: exact field value>` | `<required: exact branch ID or NONE>` | `<required: exact ordered IDs derived for this unit, not copied cluster union>` | `<required: exact ordered delta or NONE>` | `<required: exact ordered union>` | `<required: included, excluded, or predecessor-proved plus section and fixture IDs>` |

The effective capability list is the registry-ordered, duplicate-free union of
the exact lock-local member/outcome base capabilities and that exact branch's
conditional delta. The G0 cluster capability list is only an obligation
envelope and is never the base list by implication. The
singleton delegated row `ALLOC-ERROR-01` must name exactly
`SEQ-TRY-RESERVE-01;DEQUE-RESERVE-01;HEAP-RESERVE-01;HMAP-RESERVE-01;HSET-RESERVE-01;STRING-RESERVE-01`;
the lock then selects the applicable owner clusters and maps their exact branch
keys to lock-local member/outcome units.
Delegation creates no independent payload mechanism or closure. A
`BOUNDARY_EVIDENCE_ONLY` row may only be excluded as an inadmissible spelling.
A `FRAME_SCOPE_DEFERRED` row may proceed only through its exact separately
locked and hostile-reviewed frame/ABI authority. Neither state is a safe
ordinary-library derivation.

The lock must preserve the generated role carrier sets and member partitions.
Active extract, splice, and filter rows consume their existing `BR-STORED`
obligation for live `RangeBounds` state where present and for retained callable,
replacement-source, or cursor state; they may not be restated as a weaker
conditional branch. Hash-set relations/algebra and exact trait branches must
state whether stored `BuildHasher` or caller-owned mutable `Hasher` state is
preserved, reborrowed, used lazily, retained by `Extend`/`Collect`, or destroyed
by owning `IntoIterator`. Non-hash siblings may not inherit that state or its
cost. Every callable row freezes zero-or-more, zero-or-one, or exactly-once
invocation and exactly-one normal-route destruction. Only
`VIEW-SORT-01::CACHED_KEY_BORROW_STATE` may budget a retained cached-key array;
a `KEY_RESULT_BORROW_STATE` does not authorize one.

Every instantiated lock must treat each runtime behavior call as effectful by
default, including calls through shared receivers. For every selected member,
freeze the exact behavior-effect and result-provenance relations over the
joint pre-state and post-state: nonconsumed outer owners remain valid;
surviving leaves keep roots; ended leaves end once; moved leaves have one
destination and end at the source before destination liveness; new leaves are
relation-authorized; and no unique leaf is simultaneously live in two roles.
Temporary reborrows, receiver or field addresses, container storage, and call
frames mint no root unless the result relation identifies actual returned
storage. Repeated calls use the preceding post-state. Unless a stronger
relation or verified-body proof is locked, candidates and optimizers must not
assume purity, idempotence, repeatability, leaf-map preservation, call elision,
duplication, common-subexpression elimination, fusion, or reordering. Logical
law failure may affect logic, results, complexity, or refinement only and must
never weaken safety or fact firewalls. The lock must measure the direct
monomorphized zero-runtime-tax path and reject hidden dynamic dispatch.

An excluded conditional branch, unresolved delegation, boundary-evidence row,
or frame-scope deferral must contribute zero additional fields, metadata bytes,
checks, branches, allocations, generated-code paths, payload traffic, or new
fact dependencies from its stronger scope to the frozen region-free/borrow-free
default shape. An included branch may pay only for its stronger contract, and
its proof state must erase when derivable from existing fields. The performance
matrix must contain a facts-off and structural control that can detect either
conditional-cost leakage or a hidden generic fallback.
Any unresolved `DEFERRED_BRANCHES`,
`DELEGATED_TO_FAMILY_BRANCHES`, `BOUNDARY_EVIDENCE_ONLY`, or
`FRAME_SCOPE_DEFERRED` row forbids unrestricted `E` or `P`.
No G0 coverage cluster, derivation-matrix row, or overlay key alone can support
`E`, `P`, candidate construction, or a scored experiment. Those actions remain
blocked until every key in each routed cluster's complete evidence audit domain
has an independently derived target set, every applicable
`(evidence_identity, target_id)` pair has exactly one terminal disposition, no
non-applicable pair has a terminal, and every refined pair has exact
member/outcome semantics, capabilities, costs, algorithms, canaries, and
endpoints frozen by this lock. Any exclusion blocks every claim named by its
disposition.

## 5. Exact caller semantics

Each exact member operation is frozen as:

```text
pre-state + input ownership + behavior parameters
    -> post-state + result ownership + failure/destruction effects
```

| Member contract ID | Outcome ID | Exact declaration and implementation evidence keys | Outcome class and trigger | Pre-state | Input ownership and borrows | Behavior parameters, effects, and call count | Result and post-state | Invalidation | Error or trap behavior | Normal-exit/drop/abandonment behavior | Complexity, allocation, and resource ceiling | Layout, identity, order, and range guarantees | Exact capability IDs and payload branches |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `<required: stable member_contract_id>` | `<required: stable outcome_id>` | `<required: exact canonical declaration and concrete-impl keys>` | `<required: normal success, recoverable failure, checked trap, abandonment, destruction, or other exact class plus trigger>` | `<required: exact state predicates>` | `<required: modes, provenance, and offered affine values>` | `<required: static behavior, leaf effects, exact invocation partition, or none>` | `<required: exact result ownership, provenance, and state>` | `<required: exact references, cursors, facts, and identities invalidated>` | `<required: exact error ownership or checked-trap conditions>` | `<required: every owner, live value, allocation, callable, and protocol-state disposition>` | `<required: conditional asymptotic and enforceable structural resource contract>` | `<required: exact observable guarantees>` | `<required: exact ordered IDs and overlay keys for this unit>` |

Every `member_contract_id` must enumerate all normal, recoverable-failure,
checked-trap, abandonment, and destruction outcomes that can affect ownership,
provenance, invalidation, cleanup, behavior calls, allocation, or cost. The
completed table must distinguish trapping and recoverable forms, reserve and
shrink families, drain and clear, resize and truncate, strict and lossy text
conversion, borrowed and allocating text transforms, ordered and unordered
removal, shared/unique/owning traversal, eager and lazy protocols, iterator
consumers, Rc and RefCell branches, trait members and concrete implementation
classes, deep clone and shared identity, and every other difference preserved by
the complete applicable evidence. Candidate construction and scored execution are invalid
until this table and its complete evidence-key mapping pass exact-hash hostile
review.

## 6. Abstract state and preservation proof

The lock must define an abstract state without requiring candidates to share a
runtime representation.

```text
State_F = <required: allocation, live-set, ownership, borrow, identity,
                    refinement, behavior, and fact components>

Valid_F(state) = <required: complete invariant>
```

| Proof item | Required family-local statement | Evidence form |
|---|---|---|
| Initialization theorem | `<required: every constructor establishes Valid_F>` | `<required: proof artifact and fixture IDs>` |
| Transition preservation | `<required: every operation preserves Valid_F on every normal result>` | `<required: proof artifact and fixture IDs>` |
| Slot/owner ledger | `<required: live iff readable/droppable; exactly one owner per affine value>` | `<required: machine-checkable state accounting>` |
| Refinement preservation | `<required: producers, preserving mutations, and invalidators>` | `<required: proof and negative fixtures>` |
| Destruction theorem | `<required: exactly live payloads dropped and allocations freed once>` | `<required: drop-counter and state proof>` |
| Ordinary-library derivability | `<required: public checked dependency derivation>` | `<required: sealed and ordinary-library witness plan>` |
| Protected-path theorem | `<required: B-FIX and B-P2 unchanged under every candidate>` | `<required: exact structural and code-shape evidence>` |

Temporary invalidity is admissible only inside one checked transition or a
scoped protocol whose every normal exit proves `Valid_F`. No candidate may
expose dead/spare/moved-from bytes as `T`, unchecked length or occupancy
authority, or split metadata/payload mutation.

## 7. Ownership, normal exits, abandonment, and destruction

### 7.1 Ownership transition ledger

| Transition ID | Source owners before | Destination owners before | Source state after | Destination state after | Returned ownership | Drop count | Failure ownership | Required traffic model |
|---|---|---|---|---|---|---|---|---|
| `<required: transition ID>` | `<required: exact owners>` | `<required: exact owners or dead storage>` | `<required: live/dead state>` | `<required: live/dead state>` | `<required: exact returned values>` | `<required: exact zero/one counts>` | `<required: preserved or returned owners>` | `<required: direct compulsory transfers used for structural comparison>` |

Clone is recorded separately from relocation. No row may rely on hidden
`Copy`, `Clone`, `Default`, placeholder construction, or a writer-readable hole.

### 7.2 Exit matrix

| Exit class | Applicable operation/protocol points | Required owner state | Required payload disposition | Required allocation disposition | Exact fixture IDs |
|---|---|---|---|---|---|
| Fallthrough | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| `return` | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| `break` | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| `give` | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| `try` propagation | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| Recoverable capacity/allocation failure | `<required: points>` | `<required: frozen failure state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| Behavior/callback stop or error | `<required: points>` | `<required: Valid_F post-state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| Affine protocol abandonment | `<required: every token/cursor/guard>` | `<required: valid base-owner state>` | `<required: exact>` | `<required: exact>` | `<required: IDs>` |
| Owner destruction | `<required: every owner form>` | `<required: owner dead>` | `<required: exactly-once live payload drop>` | `<required: exactly-once release>` | `<required: IDs>` |
| Trap | `<required: every trap site>` | No recoverable post-state | No cleanup required; no invalid read or other undefined behavior before abort | No cleanup required | `<required: IDs>` |

For every lazy cursor, the exit matrix separately records construction,
yielding, logical exhaustion (`next` returns `None`), any permitted repeated
terminal call, and destruction or consuming close from unused, partial, and
exhausted states. Logical exhaustion does not by itself release a source borrow,
destroy retained state, or run deferred repair unless the frozen API consumes
or invalidates the cursor at that transition. It may retire or replace a named
subcursor or epoch only when the exact concrete transition says so. Proven last
use may release only pure source-borrow authority after every incompatible
yielded child ends and only when no repair, owned-state destruction, or
allocation disposition remains; it never substitutes for those effects.
Base-owner reuse requires cursor authority and every incompatible yielded child
to end and every required repair, destruction, and allocation disposition to
complete.

Affinity is not a must-finish proof. For every protocol value, the lock must
freeze one exact policy: eager valid commit before abandonment, statically
exact-use completion on every normal path, or compiler-owned derived cleanup.
The selected policy is candidate-visible semantics and receives META-5,
soundness, structural-cost, and hostile-review accounting. Writer discipline is
not a policy.

### 7.3 Destruction and recursive bounds

| Owner/state class | Live-set authority | Drop traversal | Partial-state rule | Recursive bound | Allocation release | Artifact visibility |
|---|---|---|---|---|---|---|
| `<required: owner/state class>` | `<required: non-forgeable authority>` | `<required: exact order and coverage>` | `<required: live-only cleanup>` | `<required: termination and stack/resource bound>` | `<required: exactly-once rule>` | `<required: elaborated operation/report form>` |

## 8. Failure and commitment protocol

| Failure ID | Operation and preparation step | Failure class | State before possible failure | Commitment point | Error result | Original-owner state | Offered-affine-input state | Rollback proof | Fault-injection fixture |
|---|---|---|---|---|---|---|---|---|---|
| `<required: stable failure ID>` | `<required: exact point>` | `<required: checked arithmetic, recoverable allocation, behavior error, trap, or current TCB-level OOM>` | `<required: exact state>` | `<required: first destructive step>` | `<required: exact result or abort>` | `<required: preserved/transformed state>` | `<required: retained, returned, or consumed state>` | `<required: proof or infallible post-commit remainder>` | `<required: fixture ID>` |

The lock must keep checked capacity overflow, recoverable allocator failure,
current OOM handling, callback error, and trap behavior distinct. If recoverable
allocation is outside the family scope, candidates may not simulate or claim it.

## 9. Borrow, provenance, cursors, and behavior

### 9.1 Access ledger

| Access ID | Root/provenance | Borrow or logical-handle form | Lifetime/epoch | Disjointness proof | Mutations blocked | Invalidation event | Escape rule | Runtime cost |
|---|---|---|---|---|---|---|---|---|
| `<required: stable access ID>` | `<required: exact root>` | `<required: exact access authority>` | `<required: exact duration/version>` | `<required: static or checked dynamic proof>` | `<required: exact list>` | `<required: exact event>` | `<required: accepted/rejected escapes>` | `<required: metadata, checks, and branches>` |

Physical references and logical handles remain distinct. A logical cursor must
revalidate the identity and occupancy promised by its contract; it does not
inherit physical address stability.

### 9.2 Behavior and refinement ledger

| Behavior/refinement ID | Callable or predicate contract | State/effects | Direct-call guarantee | Establishing transition | Preserving transitions | Invalidators | Broken-law containment | Partial-failure cleanup |
|---|---|---|---|---|---|---|---|---|
| `<required: stable ID>` | `<required: exact semantics>` | `<required: ownership, borrows, and effects>` | `<required: lowering/code-shape requirement>` | `<required: checked producer>` | `<required: exact list>` | `<required: exact list>` | `<required: logic-error boundary that cannot become memory unsafety>` | `<required: exact ownership/drop rule>` |

Derived clone, comparison, hashing, formatting, or diagnostics may never inspect
dead or uninitialized fields. Behavior-law inconsistency may affect results or
complexity only within the frozen containment boundary.

## 10. Candidate and reference set

The template supplies no candidate. The instantiated lock must freeze the
complete comparison set before any implementation begins.

| Candidate-set field | Required locked value |
|---|---|
| Candidate count and IDs | `<required: complete finite set>` |
| Inclusion derivation | `<required: why each candidate could close the frozen contracts>` |
| Excluded plausible mechanisms | `<required: exact list and evidence-backed exclusion reason>` |
| Pareto dimensions | `<required: rules, checker/TCB state, facts/paths, spellings, runtime costs, code size, and protected-path tax>` |
| Completeness challenge | `<required: independent hostile argument that no materially distinct admissible candidate was omitted>` |

| Candidate ID | Public semantic contract | Mechanism description | Public and trusted dependencies | Reference algorithm/derivation | Claimed proof dimensions | Expected structural costs | Cross-family exposure | Construction assignment |
|---|---|---|---|---|---|---|---|---|
| `<required: stable candidate ID>` | `<required: exact shared contract>` | `<required: mechanism stated without implementation code>` | `<required: exact allowlist and trusted paths>` | `<required: exact algorithm specification and source>` | `<required: PD-* and capability IDs>` | `<required: preregistered cost hypotheses>` | `<required: exact exposed state/facts/paths>` | `<required: author/model/tool assignment>` |

Candidate inclusion requires a complete derivation sketch for every mandatory
operation. A candidate is structurally rejected before timing if it requires an
unsupported mandatory path, asymptotic regression, per-element allocation for
a contiguous contract, generic payload construction for spare capacity, hidden
whole-value Copy/Clone, unbounded tombstone/history growth outside the contract,
writer-visible raw initialization/occupancy state, candidate-specific compiler
recognition, or a tax forbidden by B-FIX/B-P2.

## 11. Candidate construction and correction protocol

### 11.1 Construction freeze

| Field | Required locked value |
|---|---|
| Candidate authors/models/tools | `<required: exact identities, versions, settings, and assignments>` |
| Candidate-visible materials | `<required: exhaustive paths, prompts, contracts, examples, and exclusions>` |
| Implementation order | `<required: randomized, balanced, or fixed order with contamination argument>` |
| Shared scaffolding | `<required: exact permitted common code and ownership>` |
| Candidate-specific scaffolding | `<required: exact permitted differences>` |
| Time budget | `<required: exact per-candidate budget and clock rule>` |
| Token/interaction budget | `<required: exact per-candidate budget>` |
| Repair budget | `<required: exact attempts, diagnostics visibility, and stop rule>` |
| Tuning budget | `<required: exact permitted training-only tuning>` |
| Build and test access | `<required: exact commands and visible outputs>` |
| External-service disclosure | `<required: exact authorization and materials permitted, or prohibited>` |
| Failure/disqualification rule | `<required: exact symmetric rule>` |
| Audit log | `<required: append-only event schema and artifact location>` |

### 11.2 Correction classes

| Correction class | Before Candidate Freeze B | After Candidate Freeze B | Symmetry requirement | Reopen consequence |
|---|---|---|---|---|
| Semantic or contract change | Prohibited without reopening Lock A | Invalidates Freeze B and reopens Lock A | Apply to all candidates after refreeze | Full rerun |
| Candidate-set or algorithm change | Prohibited without reopening Lock A | Invalidates Freeze B and reopens Lock A | Reconstruct under refrozen protocol | Full rerun |
| Gating soundness-fixture change | Reopens Lock A | Invalidates Freeze B and reopens Lock A | Apply identically | Full rerun |
| Scored workload, endpoint, target, allocator, threshold, or selection change | Reopens Lock A | Invalidates Freeze B and reopens Lock A | Apply identically | Full rerun |
| Compromised held-out | Rotate within frozen contract after reopening the custody record | Invalidates Freeze B | No candidate sees replacement before new freeze | Held-out rerun |
| Candidate implementation defect | `<required: allowed correction budget and classification>` | `<required: normally diagnostic-only or exact preregistered rule>` | `<required: symmetric opportunity>` | `<required: exact consequence>` |
| Non-semantic clerical correction | `<required: independently verifiable rule and append-only explanation>` | `<required: whether Freeze B remains valid>` | No semantic/scoring effect | `<required: exact consequence>` |

No post-result rescue, discretionary tuning, selective diagnostic disclosure,
or asymmetric correction is permitted.

## 12. Soundness attack and canary freeze

Every applicable attack below receives exact positive and negative fixture IDs,
expected diagnostics or runtime results, and an inapplicability proof where
excluded. A fixture description without frozen bytes or a generator digest is
not a gate.

| Attack class | Required instantiated cases | Exact fixtures | Expected verdict/result | Proof obligation | Applicability reason |
|---|---|---|---|---|---|
| Construction cardinality | Underfill, overfill, zero, one, boundary capacity, repeated completion | `<required: IDs and hashes>` | `<required: exact>` | G-1, G-2, G-4 | `<required: reason>` |
| Offered affine input | Rejected insert/push, duplicate, checked-capacity failure, recoverable allocation failure | `<required: IDs and hashes>` | `<required: exact returned/preserved owner>` | G-3, G-8 | `<required: reason>` |
| Move-out and ownership | Move-after-pop/remove, nested affine payload, replace, swap, double move, double drop | `<required: IDs and hashes>` | `<required: exact>` | G-2, G-3, G-6 | `<required: reason>` |
| Borrow invalidation | Growth, relocation, delete, clear, shrink, owner drop under live element/slice/entry borrow | `<required: IDs and hashes>` | `<required: rejection>` | G-9 | `<required: reason>` |
| Partial protocols | Partial construction, clone, compaction, drain, retain, sort scratch, early return, token abandonment | `<required: IDs and hashes>` | `<required: valid owner and exact disposition>` | G-4 through G-6 | `<required: reason>` |
| Failure atomicity | Every arithmetic, allocation, behavior, and preparation failure point | `<required: IDs and hashes>` | `<required: exact pre/post ownership>` | G-8 | `<required: reason>` |
| Identity and ABA | Stale, cross-owner, relocated, cleared, shrink/regrow, reused, small-width exhaustion, silent-wrap attempt | `<required: IDs and hashes>` | `<required: exact contract result>` | G-10, G-11 | `<required: reason>` |
| Metadata/payload coherence | Forged live/full state, control/payload disagreement, stale version, wrong owner, mutation after proof | `<required: IDs and hashes>` | `<required: rejection or retained check>` | G-2, G-13 | `<required: reason>` |
| Dynamic disjointness | Duplicate positions, overlapping ranges, alias-equivalent handles, valid distinct positions | `<required: IDs and hashes>` | `<required: exact rejection/acceptance>` | G-9 | `<required: reason>` |
| Cursor and entry escape | Escape, use after invalidation, owner mutation while live, early abandonment | `<required: IDs and hashes>` | `<required: exact>` | G-4, G-5, G-9 | `<required: reason>` |
| Traversal composition | Nested stateful adapters, two-input exhaustion, advisory size-hint lies, early stop/error/abandon, and owning affine remainder | `<required: IDs and hashes>` | `<required: exact order, progress, borrow, owner, and drop result>` | G-3 through G-6, G-9, G-14 | `<required: reason>` |
| Recursive layout/drop | Inline cycle, adversarial depth, partial node construction, bounded-stack destruction | `<required: IDs and hashes>` | `<required: exact>` | G-6 | `<required: reason>` |
| Behavior containment | Inconsistent equality/hash/order, stateful behavior failure, clone failure | `<required: IDs and hashes>` | `<required: contained result without representation corruption>` | G-12 | `<required: reason>` |
| Layout and capacity | 32-bit/64-bit limits, zero-sized payload where supported, alignment, object-size ceiling, overflow | `<required: IDs and hashes>` | `<required: exact>` | G-7, G-8 | `<required: reason>` |
| Speculative access | Non-live SIMD lane, masked result, backend speculation across state check | `<required: IDs and hashes>` | `<required: no invalid load>` | G-13 | `<required: reason>` |
| Facts on/off | Every earned and unearned fact site | `<required: IDs and hashes>` | `<required: semantic identity and exact check/code-shape expectation>` | G-13 | `<required: reason>` |
| Ordinary-library privilege | Hidden intrinsic, standard-library-only transition, recognizable source name, forbidden dependency | `<required: IDs and hashes>` | `<required: rejection>` | G-14 | `<required: reason>` |
| Protected baseline tax | Each candidate compiled with B-FIX and B-P2 | `<required: IDs and hashes>` | `<required: exact no-tax result>` | G-15 | Always applicable |

The lock must identify which fixtures are gating, scored, or diagnostic before
candidate construction. A post-lock change to a gating or scored fixture
reopens the lock.

## 13. Optimizer fact-channel freeze

List every proposed or consumed fact, including state facts that authorize
payload access even if no backend check is removed. `NONE` is acceptable only
with a proof that no candidate or control produces or consumes a new fact.

| Field | Required locked value |
|---|---|
| Fact ID and schema version | `<required: stable ID>` |
| Exact proposition | `<required: quantified machine-checkable statement>` |
| Owning root | `<required: allocation/abstraction and provenance>` |
| Producer | `<required: checked transition or verified proof>` |
| Preconditions | `<required: prior state, ownership, borrow, arithmetic, and refinement conditions>` |
| Scope/version | `<required: dominance, region, owner, and epoch>` |
| Consumers | `<required: checker/backend operations authorized>` |
| Invalidators | `<required: exhaustive mutation and ownership list>` |
| Transfer rule | `<required: move, borrow, call, return, branch, and monomorphization behavior>` |
| Speculation rule | `<required: exact load/check speculation boundary>` |
| Facts-off semantics | `<required: identical acceptance, result, traps, and valid-memory accesses>` |
| Artifact evidence | `<required: surfaced producer, dependency identity, consumers, and invalidation report>` |
| Negative canaries | `<required: exact fixture IDs and hashes>` |
| Hostile review scope | `<required: independent reviewer and exact-hash record>` |

No control byte, length, generation, refinement, or version may independently
authorize payload access unless this schema proves its coherence. Payload
validity must dominate the load, not merely use of the load's result.

## 14. Performance preregistration

### 14.1 Structural rejection before timing

For each candidate, freeze pass/fail thresholds for:

- required operation and ownership/failure coverage;
- asymptotic bounds;
- allocation count and per-element-allocation prohibition;
- intermediate-collection and adapter-allocation prohibition for applicable
  composed traversal contracts;
- initialized spare-capacity bytes and hidden payload construction;
- compulsory payload moves versus the direct-transition model;
- metadata and historical/retired state growth;
- checks, branches, fact consumers, and indirection;
- code size, monomorphization growth, and static/indirect behavior calls;
- adapter-state size, source passes, callback count/order, and direct-call
  lowering for applicable composed traversal contracts;
- writer-visible or privileged state access; and
- B-FIX/B-P2 no-tax identity.

| Structural endpoint | Unit and collection method | Exact input matrix | Pass/reject rule | Tie to caller contract |
|---|---|---|---|---|
| `<required: endpoint>` | `<required: exact method>` | `<required: payload/trace/target cells>` | `<required: exact preregistered rule>` | `<required: contract ID>` |

### 14.2 Payload, trace, target, and environment matrix

| Matrix dimension | Required locked value |
|---|---|
| Payload semantic classes | `<required: Copy, affine, nested-resource, behavior, and scoped exclusions as applicable>` |
| Payload sizes/alignments | `<required: exact target-derived cells>` |
| Initial sizes/capacities | `<required: exact cells including boundaries>` |
| Operation distributions | `<required: exact realistic and adversarial traces>` |
| Load factors/topology states | `<required: exact cells where applicable>` |
| Failure injections | `<required: every recoverable point and schedule>` |
| Targets and `DataLayout` | `<required: exact triples, CPU/features, and layouts>` |
| Allocator | `<required: exact allocator, configuration, and failure policy>` |
| OS/hardware isolation | `<required: machine, power, affinity, thermal, and noise controls>` |
| Toolchain and flags | `<required: exact commands and digests>` |
| Random seeds and generators | `<required: exact seeds, generator versions, and hashes>` |

The matrix must contain no post-result cells. Any pilot used to set it is
training evidence and must be disclosed symmetrically before the lock.

### 14.3 Same-shape attribution control

The same-shape control isolates language/checker/codegen cost from a container
or algorithm choice.

| Control field | Required locked value |
|---|---|
| Shared observable contract | `<required: exact member_contract_id and outcome_id set>` |
| Algorithm identity | `<required: exact matched algorithm specification>` |
| Representation identity | `<required: field/layout/capacity-state equivalence rule>` |
| Capacity/growth policy | `<required: exact matched policy>` |
| Allocator | `<required: exact matched allocator and calls>` |
| Payload and traces | `<required: exact shared matrix cells>` |
| xlang facts-on build | `<required: exact command and expected fact accounting>` |
| xlang facts-off build | `<required: exact command and retained-check expectation>` |
| Rust/reference build | `<required: exact source, version, command, and safety boundary>` |
| Structural equivalence gate | `<required: allocations, bytes, moves, branches, calls, and layout conditions>` |
| Primary attribution endpoint | `<required: endpoint and direction>` |
| Secondary endpoints | `<required: endpoints and interpretation>` |

A same-shape mismatch is a structural result, not permission to attribute the
difference to language checking.

### 14.4 End-to-end contract control

The end-to-end control compares the canonical candidate route with the exact
unmodified idiomatic Rust 1.97 route for the same caller contract.

| Control field | Required locked value |
|---|---|
| Caller contract | `<required: exact member_contract_id and outcome_id set plus equivalence oracle>` |
| Canonical xlang route | `<required: candidate-selection result that would supply this route>` |
| Rust 1.97 route | `<required: exact stable API and unmodified source authority>` |
| Permitted adapters | `<required: contract-only harness adapters applied symmetrically>` |
| Inputs and traces | `<required: exact immutable cells>` |
| Primary endpoint | `<required: endpoint and direction>` |
| Structural context | `<required: allocations, traffic, metadata, and code-shape report>` |
| Claim wording | `<required: bounded statement if the endpoint passes>` |
| Failure wording | `<required: bounded statement if it ties, loses, or is inconclusive>` |

One-to-one API or representation parity is not required; caller-observable
contract parity is.

### 14.5 Measurement and selection rule

| Field | Required locked value |
|---|---|
| Warmup and sample schedule | `<required: exact balanced/randomized schedule>` |
| Repetitions and stopping rule | `<required: exact preregistered rule>` |
| Raw-sample retention | `<required: immutable format, path, and hash procedure>` |
| Exclusion rule | `<required: objective preregistered exclusions; no discretionary outliers>` |
| Confidence method | `<required: interval/test method and confidence level>` |
| Primary endpoint | `<required: one endpoint and aggregation>` |
| Non-inferiority gates | `<required: endpoint-specific margins and direction>` |
| Benefit gates | `<required: endpoint-specific margins and direction>` |
| Secondary endpoints | `<required: exact hierarchy and claim limits>` |
| Lexicographic tie breaks | `<required: exact ordering>` |
| Crossover rule | `<required: static specialization/dispatch policy or explicit exclusion>` |
| Unique-survivor rule | `<required: exact selection function>` |
| No-selection rule | `<required: exact outcome when no unique survivor exists>` |
| Multiple-comparison handling | `<required: exact correction or fixed hierarchy>` |

Non-inferiority alone cannot select a winner. If the frozen function yields no
unique survivor, the outcome is `NO-SELECTION`; it does not authorize several
equivalent writer-facing routes.

### 14.6 Required structural and measured outputs

Record, per frozen cell where applicable:

- allocations and frees;
- requested, live, high-water, retained-history, peak, and transient bytes;
- initialized, touched, copied, cloned, and moved payload bytes;
- metadata bytes per live element and per capacity;
- exact drop counts and drop-scan traffic;
- bounds, alias, occupancy, generation, refinement, and behavior checks;
- branches, indirect calls, vector width, instructions, and code size;
- load factor, probe count, fragmentation, cache/TLB events, and reliable
  topology-specific counters;
- throughput or latency endpoints with the frozen distribution; and
- facts-on/facts-off producer and consumer accounting.

Every unavailable counter needs an exact reason and a preregistered substitute
or excluded claim.

## 15. Protected baseline no-tax gates

`B-FIX` and `B-P2` run for every candidate, including candidates that do not
use either topology. The capability registry maps these protections to
`NT-FIXED` and `NT-P2`.

### 15.1 B-FIX: fixed fully initialized Copy buffer

The future lock must pin the exact existing source and expected artifacts, then
require all of the following under every candidate compiler/configuration:

- the same two-word owner, fixed length, one allocation, and checked index
  contract;
- no capacity, occupancy, initialization, generation, sharing, identity, or
  policy field;
- no new branch, check, allocation path, drop scan, runtime mode, or indirect
  call;
- unchanged accepted source and diagnostics; and
- unchanged relevant raw IR, optimized hot body, traps, alias metadata,
  vectorization, call set, layout, and structural counters except for an exact
  preregistered toolchain-normalization allowlist.

| B-FIX field | Required locked value |
|---|---|
| Source and test identity | `<required: exact paths and SHA-256>` |
| Target/toolchain | `<required: exact identities>` |
| Layout/ABI oracle | `<required: exact expected fields, sizes, and alignments>` |
| Code-shape oracle | `<required: normalized IR/assembly/call/trap expectations>` |
| Structural oracle | `<required: allocation, metadata, check, and branch expectations>` |
| Candidate comparison rule | `<required: byte/exact-normalized equality rule>` |
| Failure consequence | Reject the candidate before scoring |

### 15.2 B-P2: append-only SoA/index pool

The future lock must pin the existing append-only source and measured path,
then require:

- non-reused indices and the existing append-only identity contract;
- no generation, retirement, recycling, sparse-state, reference-count,
  shared-ownership, relocation, cross-pool-provenance, or runtime-policy field
  or branch;
- no new per-access metadata load, identity check, indirection, or call; and
- unchanged relevant source acceptance, layout, optimized access body, bounds
  behavior, alias facts, vectorization, call set, and structural counters except
  for an exact preregistered toolchain-normalization allowlist.

| B-P2 field | Required locked value |
|---|---|
| Source and test identity | `<required: exact paths and SHA-256>` |
| Target/toolchain | `<required: exact identities>` |
| Layout/identity oracle | `<required: exact columns, handle/index width, and non-reuse contract>` |
| Code-shape oracle | `<required: normalized IR/assembly/check/call expectations>` |
| Structural oracle | `<required: metadata, branch, load, and code-size expectations>` |
| Candidate comparison rule | `<required: byte/exact-normalized equality rule>` |
| Failure consequence | Reject the candidate before scoring |

A runtime feature flag that selects fixed versus growable or append-only versus
recyclable behavior fails these controls if the protected path pays any field,
branch, check, or code-shape cost.

## 16. Ordinary-library and held-out witness custody

### 16.1 Witness applicability

| Witness ID | Role | Why implicated | Exact visible contract | Dependency allowlist | Forbidden dependencies | Gate stage |
|---|---|---|---|---|---|---|
| `<required: B, W, H, or promoted O witness ID>` | `<required: role>` | `<required: capability/family dependency>` | `<required: exact frozen contract reference>` | `<required: exact public dependencies>` | `<required: exact exclusions>` | `<required: structural, soundness, scored, or composition stage>` |

Every implicated W witness must compile as an ordinary external xlang library
using the same public checked capabilities as unrelated code. A sealed library
witness cannot substitute for it.

### 16.2 Held-out custody

| Custody field | Required locked value |
|---|---|
| Applicable held-outs | `<required: exact subset of H-FLATSET, H-STORE, H-LRU, H-IPQ, and any owner-approved replacement>` |
| Visible contract and allowlist | `<required: exact frozen text and hash>` |
| Custodian | `<required: identity independent of candidate construction>` |
| Hidden source location | `<required: non-candidate-visible custody location>` |
| Hidden source hash | `<required: externally recorded SHA-256>` |
| Hidden tests/oracles hash | `<required: externally recorded SHA-256>` |
| Hidden trace/generator hash | `<required: externally recorded SHA-256>` |
| Candidate exclusion proof | `<required: access and disclosure log>` |
| Disclosure point | After Candidate Freeze B records every candidate hash |
| Scoring access | `<required: exact process and identities>` |
| Leak detection | `<required: audit method>` |
| Rotation rule | `<required: replacement procedure within unchanged contract and dependency budget>` |
| Compromise consequence | Diagnostic-only result; reopen custody record and Candidate Freeze B |

No plaintext repository artifact may be described as training-excluded. A
held-out implementation cannot train candidates, tune thresholds, induce
compiler recognition, or receive a candidate-specific correction.

## 17. META-5 and trusted-delta ledger

Every candidate-visible or candidate-required language/toolchain delta is
cumulative. A row is required even when the candidate claims the delta is
zero.

| Delta ID | Candidate(s) | Public spelling | Normative rule | Grammar | Type/ownership/borrow | Effect/exit/drop | Diagnostic | Checker proof state | Trusted fact/path | Lowering/codegen | Artifact/reporting | Tests and hostile review | Derivation and necessity | Protected-baseline effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `<required: stable delta ID or explicit ZERO-DELTA row>` | `<required: exact candidates>` | `<required: exact surface or none>` | `<required: exact spec delta>` | `<required: productions or none>` | `<required: exact obligations>` | `<required: exact obligations>` | `<required: exact diagnostics>` | `<required: exact state/proof changes>` | `<required: exact TCB/fact/path>` | `<required: exact lowering>` | `<required: exact surfaced evidence>` | `<required: exact fixtures and review>` | `<required: why existing checked capabilities cannot derive the contract>` | `<required: B-FIX/B-P2 proof>` |

The ledger must count normative rules, checker and TCB state, trusted facts and
paths, writer spellings, runtime metadata/branches, code size, and tax on
protected baselines. Primitive count alone is not a minimization objective.

## 18. E0.1 disposition

Every lock must state whether it intersects E0.1. A non-intersection requires a
contract and capability proof; a family name alone is insufficient.

| E0.1 input | Required disposition |
|---|---|
| Current fixed buffer control | `<required: retain as B-FIX with exact authority>` |
| Declarative Copy route | `<required: retain, revise, supersede, or NOT-IMPLICATED with reason>` |
| Affine fixed builder | `<required: retain as control, revise, supersede, or NOT-IMPLICATED with reason>` |
| Automatic structural Copy | `<required: candidate, superseded alternative, or NOT-IMPLICATED with reason>` |
| Copy-by-default/negative-affine route | `<required: preserve rejection or reopen with exact evidence>` |
| Affine fixed-storage predicate | `<required: relationship to family storage eligibility>` |
| Recursive/single-level recipe | `<required: exact disposition and grammar/effect consequence>` |
| Explicit Repeat/Clone | `<required: separate duplication contract or NOT-IMPLICATED>` |
| Per-slot builder/initialized prefix | `<required: separate construction protocol from steady state>` |
| Public raw or split uninitialized privilege | Preserve rejection; a Family Lock A cannot reopen W3 or create writer-visible unchecked state |
| Historical layout/capacity controls | `<required: retained or superseded exact fixtures>` |
| Historical numeric/statistical inputs | `<required: retain or replace before candidates; explain consequence>` |
| Capability adoption | Remains separate from xlc migration and default teaching |

If the family intersects dense storage, the lock must also enumerate every new
mandatory operation absent from E0.1 and explain the exact relation between
fixed AoS ownership, semantic Copy, and partially live affine storage.

## 19. Review, exact hashes, and Candidate Freeze B handoff

### 19.1 Lock artifact manifest

| Artifact | Exact path/identity | SHA-256 | Producer | Reviewer | Status |
|---|---|---|---|---|---|
| Instantiated Family Lock A | `<required: path>` | `<required: SHA-256>` | `<required: identity>` | `<required: independent identities>` | `<required: exact-hash PASS before construction>` |
| Fail-closed Family Lock validator and PASS record | `<required: source, executable, and result paths>` | `<required: SHA-256 per artifact>` | `<required: identity>` | `<required: independent exact-hash reviewer>` | `<required: PASS on exact lock bytes before construction>` |
| Contract tables | `<required: paths>` | `<required: SHA-256 per artifact>` | `<required: identity>` | `<required: identity>` | `<required: status>` |
| Soundness fixtures/generators | `<required: paths>` | `<required: SHA-256 per artifact>` | `<required: identity>` | `<required: identity>` | `<required: status>` |
| Performance protocol/tools | `<required: paths>` | `<required: SHA-256 per artifact>` | `<required: identity>` | `<required: identity>` | `<required: status>` |
| META-5 ledger | `<required: path>` | `<required: SHA-256>` | `<required: identity>` | `<required: identity>` | `<required: status>` |
| Holdout custody record | `<required: non-visible record identity>` | `<required: externally recorded hashes>` | `<required: custodian>` | `<required: independent auditor>` | `<required: status>` |

Required independent review scopes are:

1. census, capability, witness, E0.1, design-tree, and repository consistency;
2. ownership, initialization, exit, destruction, failure, borrow, identity, and
   fact-channel soundness; and
3. construction symmetry, structural performance, measurement, selection,
   holdout custody, staging, and claims discipline.

Every finding is repaired before a final exact-hash pass. A green repository
gate is evidence, not hostile review.

### 19.2 Candidate Freeze B handoff

After candidate construction and before any scored or held-out execution,
Candidate Freeze B must record:

- exact candidate, shared-scaffolding, compiler, specification, and test hashes;
- build commands, toolchain, target, allocator, and environment;
- construction logs and proof of budget/protocol compliance;
- training-only results and every allowed correction;
- structural-gate results;
- immutable scored inputs and scoring-tool hashes;
- held-out custody integrity and disclosure authorization; and
- the correction, invalidation, and symmetric-rerun rules inherited from this
  lock.

Candidate Freeze B may not repair a missing Lock A field. A missing or changed
semantic contract, candidate, algorithm, workload, soundness gate, fact,
threshold, endpoint, or custody rule reopens Lock A.

## 20. Freeze, reopening, and durability

An instantiated lock freezes only after all markers are resolved, all exact
artifacts are hashed, hostile reviews pass on those exact bytes, repository
gates pass, and the owner records the allowed next action.

The lock reopens before further candidate work when any of the following
changes or is discovered incomplete:

- caller result, ownership, invalidation, failure, trap, drop, complexity,
  layout, contiguity, identity, order, range, behavior, or resource contract;
- family role, dependency, capability applicability, witness budget, or scoped
  deferral;
- candidate mechanism, reference algorithm, construction assignment, budget,
  correction rule, or crossover rule;
- payload, trace, target, allocator, endpoint, threshold, confidence method,
  tie break, selection function, or scored/gating fixture;
- public spelling, normative rule, checker state, trusted fact/path, lowering,
  or artifact reporting;
- held-out visibility, implementation, contract, allowlist, custody, or
  compromise state;
- supposedly local state or fact proves cross-family; or
- evidence invalidates soundness, ordinary-library derivability, a protected
  no-tax result, or an asymptotic/performance premise.

After Candidate Freeze B, any reopening also invalidates that freeze and all
affected scored results. A non-semantic clerical correction may avoid reopening
only under the exact rule frozen in Section 11, with an append-only explanation
and independent verification that no acceptance, code, measurement, or claim
changes.

Durability requires one commit plus one append-only
`decision-gates.md` entry for the completed lock step. The gate entry records
the exact lock hash, review hashes and verdicts, repository verification,
authorization scope, and next prohibited actions.

## 21. Owner authorization and claims boundary

The instantiated lock must end with an owner authorization record:

| Boundary | Required locked value |
|---|---|
| Activities authorized after lock approval | `<required: exact bounded list>` |
| Activities still prohibited | `<required: candidate scoring, implementation, spec/compiler change, production adoption, xlc migration, E0.1 restart, and teaching disposition as applicable>` |
| Candidate Freeze B approval requirement | `<required: exact owner/review gate>` |
| Scored and held-out run approval requirement | `<required: exact owner/review gate>` |
| Family closure criterion | `<required: all M/W/H/B, proof, structural, measured, fact, and review conditions>` |
| Production adoption gate | Separate owner decision after family closure |
| Specification/compiler implementation gate | Separate owner decision; lock evidence alone changes no production artifact |
| xlc migration gate | Separate owner decision and workload-specific evidence |
| Default teaching gate | Separate benchmark-blind writer evidence and owner decision |
| Complete-floor claim | Prohibited until every protected B and mandatory M/W/H obligation closes |
| Whole systems-language claim | Prohibited until every deferred domain required by the owner closes |

Family closure permits only the bounded claim frozen in Section 2. It does not
inherit adjacent contracts, optional operations, excluded payloads or targets,
shared substrate approval, or a general-purpose systems-language claim.

## 22. Lock-completion record

| Completion item | Required final record |
|---|---|
| All field markers resolved | `<required: verifier result and hash>` |
| Contract/capability/witness closure | `<required: verifier result and hash>` |
| Soundness fixture freeze | `<required: verifier result and hash>` |
| Construction/correction freeze | `<required: verifier result and hash>` |
| Performance and selection freeze | `<required: verifier result and hash>` |
| B-FIX/B-P2 no-tax oracle freeze | `<required: verifier result and hash>` |
| Held-out custody freeze | `<required: auditor result and external hash record>` |
| META-5 closure | `<required: ledger result and hash>` |
| E0.1 disposition | `<required: traceability result and hash>` |
| Independent hostile reviews | `<required: exact-hash PASS records>` |
| Repository verification | `<required: commands, results, and environment>` |
| Owner authorization | `<required: exact decision and allowed next action>` |
| Durability | `<required: commit ID and decision-gates line>` |

Until every row is resolved and the exact lock receives owner approval, the
artifact remains a draft and authorizes no candidate construction or result.
