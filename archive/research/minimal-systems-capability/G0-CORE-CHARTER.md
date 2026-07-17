# Minimal Systems Capability Basis: G0-Core Research Charter

Status: owner-authorized research charter, 2026-07-14. This charter authorizes
the complete research and accounting program described below. It authorizes no
candidate or production language mechanism, compiler implementation,
specification change, pattern-catalog change, xlc migration, scored candidate
run, or default teaching change.

## 1. Research question

xlang is intended to be a general-purpose systems language. It may ship no
standard library, and it need not reproduce Rust's named types or APIs. It must,
however, let checked ordinary libraries derive the everyday capabilities that a
systems-language standard library provides, with competitive asymptotic and
structural performance and without writer-visible unsafe or container-specific
compiler privilege.

The research question is therefore:

> What Pareto-small set of checked storage, ownership, lifetime, behavior,
> boundary, and proof capabilities lets ordinary xlang libraries derive the
> required systems contracts efficiently while preserving the protected
> zero-tax paths?

"Small" does not mean the fewest primitive names. A one-item raw-memory escape
has maximal permission and proof cost and is inadmissible. Candidate bases are
compared by normative rules, checker and trusted-computing-base state, trusted
facts and paths, writer spellings, runtime metadata and branches, code size, and
tax on protected baselines. The result may be a Pareto frontier rather than a
mathematically unique minimum.

## 2. Finite external anchor

The primary completeness anchor is the stable public library surface of Rust
1.97.0, released 2026-07-09:

- release tag: `1.97.0`;
- annotated tag object: `eca4cdea45792600b4275e9d4c64fd827d575a24`;
- peeled source commit: `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`;
- reference crates: `core`, `alloc`, and `std`;
- reference documentation and source: the rustdoc and `library/` tree generated
  from that exact commit.

Rust supplies a finite, independently maintained list of caller needs. It is
not a design oracle. Rust names, traits, `Deref`, `Drop`, unsafe implementation
techniques, and concrete representations are evidence, not defaults for xlang.
The census extracts observable contracts: results and order, ownership and
invalidation, failure and destruction, complexity, contiguity and address or
identity guarantees, iteration and range behavior, allocation and memory
ceilings, behavior parameters, concurrency semantics, and platform boundaries.

Stable safe caller operations form the primary contract anchor. Stable unsafe
operations, nightly APIs, and unsafe standard-library implementation code are
recorded separately as evidence of implementation requirements or privileged
boundaries; they are never treated as an acceptable xlang surface merely
because Rust exposes or uses them.

## 3. Completeness accounting

Every stable public Rust item in scope must terminate in exactly one auditable
accounting route:

1. a coarse caller-observable coverage and obligation cluster;
2. a redundant surface form derived from another cluster;
3. a trusted platform-frame or ABI boundary whose privilege is explicitly
   priced;
4. an explicit later family lock with a named blocked claim; or
5. a documented non-goal with a first-principles reason.

No unclassified item or subsystem may disappear because it is inconvenient.
Target-specific intrinsic catalogs may be compressed by architecture and
semantic class, but their module set, counts, and digest remain mechanically
accounted. Generated impl duplication and purely documentary aliases may be
deduplicated only by a recorded rule.

The detailed first accounting pass is the sequential, unique-owner,
ordinary-library data-structure floor established by D11. The whole systems envelope is also
accounted at domain level so that passing the data-structure floor cannot be
misreported as completing concurrency, resource/FFI, custom-allocation,
async/cancellation, pin/address-sensitive, shared-ownership, Unicode-text, or
target-intrinsic capability.

The contract census records the unrestricted Rust caller demand as 276 coarse
coverage and obligation clusters. These clusters are not exact operation
contracts and are not importable closure units. The generated payload ledger
partitions all 276 clusters into 26 `ACTIVE_BR_STORED`, 138
`DEFERRED_BRANCHES`, 100 `NO_STORED_BORROW_COMPLEMENT`, nine
`BOUNDARY_EVIDENCE_ONLY`, two `FRAME_SCOPE_DEFERRED`, and one
`DELEGATED_TO_FAMILY_BRANCHES` row. A derivation row may describe a region-free
and borrow-free base route only when `PAYLOAD-SCOPE-CLASSIFICATION.tsv`
classifies that cluster and every omitted live borrow-bearing generic-payload
branch is enumerated in `PAYLOAD-SCOPE-OVERLAY.tsv` with its exact role,
condition, capability delta, blocked disposition, and reopening trigger.
The overlay contains 294 branches: 172 stored transitions, 86 borrow-bearing
owned results, and 36 retained protocol states. Exactly 139 branches require
result provenance, and 100 publicly return a borrow-bearing owner.

Payload scope includes retained behavior and protocol state, not only container
elements. Active `BR-STORED` rows for extract, splice, and filter protocols
cover their live `RangeBounds` descriptor where present and their retained
callable, replacement-source, or cursor state without a duplicate overlay row.
Hash-set relations/algebra and the exact `Index`, `IntoIterator`, `Extend`,
`Collect`, and `Cmp` branches account separately for borrow-bearing stored
`BuildHasher` or caller-owned `Hasher` state. Callable ownership is
member-partitioned as zero-or-more, zero-or-one, or exactly-once invocation,
with exactly one environment destruction on every normal route. Only
`VIEW-SORT-01` may carry the cached-key array role; ephemeral key results in
other search, selection, deduplication, or unstable-sort branches do not imply
that allocation or retained array.

Every runtime behavior call is effectful by default, including a call through
a shared receiver. Normal return preserves each nonconsumed outer owner, not a
frozen internal leaf map. The declared behavior-effect and result-provenance
relations jointly account every surviving, ended, moved, and newly live leaf;
a transferred unique leaf ends at its source before destination liveness and
is never live in both. Temporary reborrows, receiver or field addresses,
container storage, and call frames mint no root unless the result relation
names the actual returned storage. Repeated calls consume the preceding
post-state. Without a declared relation or verified-body proof, no route may
infer purity, idempotence, repeatability, leaf-map preservation, call elision,
duplication, common-subexpression elimination, fusion, or reordering. Broken
logical laws remain contained logic, result, complexity, or refinement
failures and never relax the ownership, provenance, occupancy, liveness,
disjointness, or check fact firewalls. Direct monomorphization remains the
zero-runtime-tax path.

Stored `BuildHasher S` and generated `S::Hasher H` are separate ownership and
provenance roles, but a `build_hasher` transition relates their leaf states.
The same `S` owner remains valid while its internal leaves may end, be replaced,
or move only under the declared `BuildHasher` behavior-effect relation. Each
call creates exactly one `H`; its initial leaves and the post-call leaves in
`S` jointly follow the result and behavior-effect relations. A unique leaf
transferred from `S` ends there before becoming live in `H` and is never live
in both. The same `H` owner remains valid across `Hash` and `Hasher` calls,
whose leaf transitions require the declared `Hasher` behavior-effect relation,
and is destroyed exactly once with its remaining state. Surviving leaves keep
their roots, new leaves follow their declared relations, and no leaf derives
from the call-scoped `&S`, an `S` field's address or storage, or the call frame.
The compiled root-swap and unique-transfer canaries demonstrate both permitted
transitions. Hash output may influence logical probing or results but cannot
alone authorize occupancy, liveness, uniqueness, payload access, equivalence,
or check elision. HashMap/HashSet equality is separately partitioned: it
iterates the left operand and probes only the right, so only right-hand `S` is
invoked; length mismatch and empty equality create zero `H` owners, and each
performed right-hand probe creates one. Only `Hash` implementations receive a
caller-owned `H`; other comparison branches use neither role. The overlay
independently records
whether the public API returns a borrow-bearing owner and whether a public or
internal behavior result requires `BR-RESULT`; the latter is required even
when the outer API returns only a scalar or unit.

`scope_owner_contract_ids` is the machine-readable ownership edge for a scope
decision that is not local to the row. Its only delegated row is
`ALLOC-ERROR-01`, whose exact owners are the six family-specific reserve rows.
Boundary evidence and trusted-frame scope remain attached to their exact raw or
frame evidence rows; neither is an ordinary safe-library complement or a
closure claim. A direct borrowed result is not a stored payload and remains
accounted in the base matrix through `BR-PROV` and `BR-RESULT`.

`DEFERRED_BRANCHES`, `DELEGATED_TO_FAMILY_BRANCHES`,
`BOUNDARY_EVIDENCE_ONLY`, and `FRAME_SCOPE_DEFERRED` are non-closed states. None
may receive `E` or `P` for the unrestricted Rust demand. An excluded branch
or non-closed route contributes zero additional fields, metadata bytes, checks,
branches, allocations, generated-code paths, or payload traffic from its
stronger scope to a protected region-free/borrow-free default shape. A later
lock must select every applicable payload branch key, bind it to a lock-local
exact member and outcome contract, and compute that exact unit's effective
capability set. Neither a cluster capability union nor prose similarity
authorizes an adjacent member, outcome, or payload branch.

Rust's standard library is necessary but insufficient as a generativity test.
Visible cross-ecosystem topology witnesses and four training-excluded held-out
witnesses test whether public checked mechanisms support structures that were
not prebuilt for the census.

## 4. Coverage-cluster schema and non-importability

Each cluster conservatively summarizes one or more related operations using the
following demand shape:

```text
pre-state + input ownership + behavior parameters
    -> post-state + result ownership + failure/destruction effects
```

Each row records, when applicable, the dimensions that a later family lock must
resolve:

- occupancy topology: full, dense prefix, circular or segmented, sparse,
  multi-range or transient hole;
- ownership transition: borrow, move-in, move-out, replace, relocate, clone,
  share, revoke, or destroy;
- borrow provenance and invalidation;
- exact destruction on every normal exit and the current aborting-trap rule;
- checked capacity, allocation, callback, I/O, and platform failure;
- identity, address stability, order, contiguity, and range guarantees;
- callable equality, hashing, ordering, cloning, formatting, or callback
  behavior;
- asymptotic time, allocation count, initialized and moved bytes, metadata,
  checks, branches, and memory ceilings; and
- every metadata-to-payload or state-to-access optimizer fact and its
  invalidation events.

Under D11, these rows are deliberately coarse. A row may group aliases,
convenience methods, directional variants, or members that differ in ownership,
result, failure, invalidation, complexity, cleanup, behavior calls, or allocation.
The row then preserves a conservative obligation envelope; it does not assert
that those members have one exact contract, that the union capability set is
required by each member, or that the strongest cost applies to every member.

`G0-COVERAGE-CLUSTER-REGISTRY.tsv` gives every one of the 276 rows the
machine-enforced semantic class `G0_COVERAGE_CLUSTER`, binds the exact census
and matrix row bytes, and marks the row non-importable. No row is an exact
operation-semantic unit, a Family Lock
import unit, an experiment unit, or evidence for family-level `E` or `P`.
Before any candidate construction, scored experiment, or family closure claim,
the applicable Family Lock must:

1. derive the complete evidence-key audit domain for every assigned or
   implicated cluster from `G0-COVERAGE-EVIDENCE-UNIVERSE.tsv`: all stable-safe
   surface-map keys, all D10 route keys,
   stable-unsafe evidence-map keys, concrete trait-implementation keys, and
   explicit helper or protocol selectors named by the cluster;
2. independently derive the exact applicable target set `A(e)` for every
   evidence identity `e`, then give the current family or gate exactly one
   terminal disposition if and only if it is in `A(e)`, plus one terminal for
   every additional applicable gate: refined in this lock, predecessor-proved,
   or excluded with the exact family, cluster, and whole-floor claim that it
   blocks;
3. assign a stable `member_contract_id` to every distinct member contract and
   an `outcome_id` to every distinct normal, recoverable-failure, checked-trap,
   abandonment, and destruction outcome;
4. freeze exact pre-state, input ownership, behavior effects and call counts,
   post-state, result ownership and provenance, invalidation, cleanup,
   allocation behavior, asymptotic bounds, and structural costs for each
   `(member_contract_id, outcome_id)` unit;
5. assign exact capability IDs, payload-overlay branch keys, witnesses,
   algorithms, canaries, and measurement endpoints to that unit without
   inheriting a neighboring member or a cluster-wide union; and
6. preserve non-applicable children in the audit ledger without refining,
   predecessor-proving, or excluding them in this lock, and prove globally that
   every target in every `A(e)` reaches exactly one lock-local unit or explicit
   claim-blocking disposition.

The Family Lock must also account for the complete machine-derived B/M/W/H/O
union in `G0-FAMILY-REQUIREMENT-REGISTRY.tsv`. Every B control executes against
every candidate. Every applicable M, W, and H row is required in the lock or
is excluded with exact blocked family and complete-floor claims. A predecessor
is an input and cannot discharge an owner-local role requirement. Only an O row
may use an optional-not-promoted disposition, and not when a mandatory contract
or candidate depends on it. Every required primary and `C-FAIL` cross-cut
canary maps to exact member/outcome units and frozen fixture bytes.

For family `F`, role applicability is exact: all B rows, every non-O row owned
by `F`, and only owner-promoted or mandatory-required O rows. A post-adoption
row belongs to its named gate rather than a predecessor family. Implicated
families control claim boundaries and reopening; required predecessors come
only from the registry's predecessor DAG. Only a row marked
`EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY` creates a role terminal in an
implicated lock. Success uses the indivisible
`PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED`, binding both the closure owner's
reusable unit and the current family's topology-local member/outcome/canary
units. Every affecting specification, compiler, lowering, optimizer,
toolchain, target, shared capability, and dependency byte or identity must be
unchanged; otherwise every affected current-configuration canary reruns or the
closure-owner lock reopens. Other implicated-family fields affect reopening and
claim boundaries only.

The dense lock owns the first mandatory C-ITER traversal obligation. W-PIPE is
owned by the later iteration family after dense. Both carry an
each-implicated-family rebind rule: every deque, sparse, ordered, heap,
identity, recursive, arena, text, or other implicated family must bind and test
its own topology-specific traversal units. Dense or reusable predecessor
evidence alone cannot silently close those later traversal contracts.

Before candidate construction, the instantiated lock supplies a fail-closed
validator that derives the exact current-family primary and
implicated/reopening cluster sets from `G0-CLUSTER-FAMILY-ROUTING.tsv`, keeps
predecessor proof units separate so membership never routes a downstream
cluster backwards, and derives the evidence-key, role, canary, and
member/outcome sets from the remaining frozen G0 inputs. The cluster universe is
the audit domain, not an applicability union. Concrete implementation identities
take `A(e)` from the composition of their exact `impl_key` topology row and the
closed owning-cluster operation-gate assignment in
`G0-FAMILY-GATE-VOCABULARY.md`. Direct identities and selector children require
independent exhaustive applicability ledgers. The
validator proves audit-domain equality, exactly one legal terminal per
applicable `(e, target)`, no terminal for a non-applicable target, no orphan
target, complete outcome coverage, and no unknown or duplicate item. One
target's exclusion cannot erase another target's obligation. No candidate
construction, Candidate Freeze B,
`E`/`P` claim, scoring, held-out execution, family closure, or adoption request
may proceed until the validator passes, independent exact-hash hostile review
passes, and the owner separately authorizes the next action.

Cluster routing is many-to-many audit-domain and reopening evidence only. It
forces independent exact evidence-child applicability derivation and cannot
transfer a cluster-level `E`/`P` status, capability union, cost envelope, or
closure.
Access/reborrow, ownership transition, failure, and behavior are cross-cut
dimensions bound inside each applicable family, not predecessor families that
can block dense work. Only genuine scoped later families occupy `F-*` reopening
routes.

Cross-topology clusters require independent per-evidence-child family
applicability before a Family Lock may refine or exclude any child. Concrete
trait implementers cannot inherit one cluster-primary family, and no family may
exclude one of its own topology children through a self-authored classification.
Every concrete implementation key joins exactly to
`G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv`, whose primary family/gate, predecessor
families, predecessor gates, and implicated/reopening targets are independently
pinned. The owning cluster also joins to a closed operation-gate assignment.
For each concrete relation, `A(e)` is exactly the topology primary plus its
assigned operation gate, if any. A present operation gate has the topology
primary as its one child-specific immediate predecessor target; this edge is
additive to the operation gate's complete route-level predecessor family/gate
set, which remains mandatory. The topology primary's own predecessor sets remain
separate. The two applicable targets are distinct and require independent
terminals. An ungated relation is explicitly `NONE`/`NONE`,
and neither terminal nor exclusion can erase another target's obligation.

If two declarations are exact aliases, the lock may map both evidence keys to
one `member_contract_id`, but it must prove equality across every frozen
dimension. Directional or similarly named operations are not aliases merely
because the G0 cluster groups them.

## 5. Derivation standard

An exact family operation contract is covered only when its Family Lock ledger
contains all of the following:

- an ordinary checked xlang library sketch using only an explicit dependency
  allowlist;
- a normal-exit ownership and exact-destruction argument, including early
  return, checked failure, partial construction, and abandoned affine state;
- an asymptotic argument;
- structural cost accounting for allocation, initialization, element traffic,
  metadata, checks, branches, and code size;
- fact-channel and invalidation accounting;
- applicable negative soundness canaries; and
- a status of direct, derived, unproved, gap, trusted boundary, scoped deferral,
  redundant surface, or non-goal.

Coverage is quantified over every exact `(member_contract_id, outcome_id)` in
the Family Lock, not over a G0 cluster envelope. A family-local payload
restriction may support a scoped experiment, but it is not contract closure.
The classification and overlay are obligation inputs to the derivation ledger,
not exact operation semantics: missing classification, an unmapped branch, an
unenumerated complement, or a deferred complement blocks a full-contract
success claim.

The following do not count as efficient derivations unless a family lock later
measures and admits them for the exact member and outcome contract:

- constructing generic payload values for spare capacity;
- per-element allocation for a contiguous-storage contract;
- rebuilding an entire structure for a local operation with a stronger
  required bound;
- hidden `Copy` or deep-clone requirements;
- generation, sparse-state, or recycling tax on the protected append-only path;
- standard-library-only raw privilege or a container-specific compiler opcode;
- a transition that is safe only if the writer later calls `finish`;
- a route valid only for Copy, tiny payloads, or one disclosed witness; or
- a candidate-specific compiler recognition rule.

## 6. Performance evidence model

Later family locks, not G0-Core, freeze numeric margins and scored workloads.
G0-Core freezes two required controls:

1. **same-shape attribution:** match algorithm, representation, capacity policy,
   allocator, payload, and trace against Rust where possible; compare facts on
   and off and count structural operations before timing; and
2. **end-to-end contract comparison:** compare the selected canonical xlang
   route with the unmodified idiomatic Rust 1.97.0 standard-library route under
   the same observable contract.

The first control identifies language/checking tax or benefit. The second tests
whether the canonical route is competitive for the actual caller need. A named
container may be derived from another family when it preserves the frozen
contract and cost class; one-to-one API or representation parity is not
required.

## 7. Required research artifacts

G0-Core is complete only when the repository contains:

1. this exact baseline and extraction policy;
2. a reproducible raw stable-API inventory with tool version, counts, digest,
   and deduplication rules;
3. a domain ledger accounting for the full stable `core`/`alloc`/`std` surface;
4. a coarse operation coverage and obligation census for the detailed data,
   text, iteration, ownership, and lifecycle scope, with an explicit global
   non-importability rule;
5. a capability-basis registry separating caller contracts from candidate
   language mechanisms;
6. a coarse cluster-to-capability derivation screen with gaps, structural-cost
   envelopes, fact channels, and xlang evidence;
7. an exact per-cluster stored-borrow classification and generated
   cluster/branch overlay for every restricted base route;
8. generated complete cluster evidence-key, cluster-to-family/gate, and exact
   concrete-implementation-to-topology routing relations, plus a generated
   B/M/W/H/O family-role and canary registry and typed human-readable
   family/gate/dimension authority;
9. visible cross-ecosystem witnesses, held-out dependency budgets, and scoped
   deferrals;
10. explicit traceability to the D11 registry and paused E0.1 obligations;
11. a Family Lock A template carrying exact semantics, soundness, performance,
   construction, holdout, and META-5 schemas forward; and
12. a synthesis report that states what is closed, what remains a gap, and the
    smallest defensible next family research request without selecting a
    production mechanism.

Every generated table must be reproducible from checked-in scripts and
checked-in classification data. Every manual exclusion or merge needs a reason.
The final exact artifact set receives independent hostile reviews for:

- census completeness and repository/design-tree consistency;
- ownership, initialization, destruction, failure, and fact-channel soundness;
  and
- performance contracts, derivability, staging, and claim discipline.

Findings are repaired before the final exact-hash pass. Durability remains one
commit plus one `decision-gates.md` entry per completed step.

## 8. Non-authorization and claims discipline

This research may inspect sources, generate inventories, construct proof
sketches, and run non-candidate accounting tools. It may not implement or expose
language syntax, a kernel transition, trusted fact, standard container,
candidate mechanism, benchmark candidate, or production compiler path.

Completion of G0-Core authorizes only an owner discussion of the first Family
Lock A. It does not authorize that lock, any candidate implementation, or E0.1
restart. The complete general-purpose systems-language claim remains blocked by
every explicitly deferred systems domain until its own contract and evidence
gate closes.
