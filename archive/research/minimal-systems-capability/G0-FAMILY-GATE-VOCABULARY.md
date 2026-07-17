# G0 Family, Gate, Dimension, and Route Vocabulary

Status: frozen G0 authority for owner review, 2026-07-14. This document names
accounting stages only. It selects no language mechanism, syntax, runtime
representation, compiler path, candidate, benchmark threshold, standard-library
type, or production change.

## 1. Type discipline

The prefixes are semantic types, not naming style:

- `F-*` is an independently closable capability family. It may own a Family
  Lock and may appear in a predecessor set.
- `GATE-*` is a cross-family or post-family staging gate. It may own an
  obligation only after every named predecessor family has closed. A gate is
  not itself a capability family and cannot establish a complete-floor claim.
- `DIM-*` is a proof dimension that every applicable exact member and outcome
  must rebind locally. It can never own a Family Lock, appear as a predecessor,
  or be marked closed independently.
- `ALL-FAMILY-LOCKS` is the protected-control scope. It is not a family or a
  gate; every candidate in every Family Lock executes the control.

This separation is required by the independent-proof-dimension result in
`G0-CORE-REPORT.md` Section 1 and by the staged-completeness rules in
`general-purpose-data-structure-capability-RESEARCH.md` Sections 10.1, 10.3,
and 10.6. Converting a dimension into a predecessor would invent a nonexistent
lock and permit a later family to claim that local ownership, access, failure,
or behavior work had already been closed elsewhere.

## 2. Capability-family vocabulary

| ID | Kind | Bounded meaning | Source authority |
|---|---|---|---|
| `F-DENSE` | family | Contiguous initialized-prefix storage with unique ownership, affine payload transitions, relocation, and exact live-range destruction. | Research Sections 4, 5, and 6; first lock named by G0 report Section 9. |
| `F-DEQUE` | family | Bounded-memory double-ended sequence topology with its own wrap and rebalance contract. | Research Sections 4 and 5. |
| `F-SPARSE` | family | Sparse occupancy and keyed unordered storage under explicit hash and adversary contracts. | Research Sections 4, 5, and 8.3. |
| `F-ORDERED` | family | Comparison-ordered map/set or tree topology with logarithmic and ordered-range contracts. | Research Sections 4 and 5. |
| `F-HEAP` | family | Priority-queue topology and repair contract. | Research Section 4. |
| `F-IDENTITY` | family | Recyclable stable-identity pool/handle topology, including exhaustion and retained-history accounting. | Research Sections 4 and 9. |
| `F-RECURSIVE` | family | Finite-layout uniquely owned recursive topology and bounded destruction. | Research Sections 4 and 7.1. |
| `F-ARENA` | family | Multi-value arena/slab topology with bulk reset and exact payload destruction. | Research Section 4 and W-ARENA in `WITNESS-REGISTRY.md`. |
| `F-TEXT` | family | Byte and text storage with explicit UTF-8 validity sealing and boundary-safe edits. | Research Section 4. |
| `F-ITERATION` | family | Reusable traversal composition across borrowed, unique, and owning entrances after topology-local traversal has been rebound. | Research Sections 4 and 7; W-PIPE in `WITNESS-REGISTRY.md`. |
| `F-PIN-ADDRESS` | family | Address-sensitive and pinning contracts that require a separately frozen stable-address guarantee. | Research Sections 5 and 7.1; G0 report later-family boundary. |
| `F-SHARED` | family | Shared ownership, weak ownership, and last-owner transitions. | G0 report later-family boundary. |
| `F-DYNAMIC-BORROW` | family | Runtime-checked interior-borrow ownership and guard topology. | G0 report later-family boundary. |
| `F-UNICODE` | family | Complete Unicode semantics beyond UTF-8 storage validity and byte-boundary safety. | G0 report later-family boundary. |
| `F-TYPE-IDENTITY` | family | Runtime type-identity and checked downcast contracts. | Rust census deferred-family evidence. |
| `F-ALLOC` | family | User-visible allocator policy and allocation-service contracts beyond the reviewed default-heap frame. | Research Section 8; G0 report later-family boundary. |
| `F-ABI` | family | Checked ABI and foreign-resource boundary contracts; raw Rust spelling is not imported. | Research Section 8; G0 report later-family boundary. |

`F-IDENTITY` means object/handle identity. `F-TYPE-IDENTITY` means dynamic type
identity. They are deliberately distinct.

## 3. Cross-family gate vocabulary

| ID | Kind | Bounded meaning | Required stage |
|---|---|---|---|
| `GATE-KEYED-ENTRY-CROSS-FAMILY` | gate | Entry/occupied/vacant behavior shared by sparse and ordered keyed stores. | After the exact sparse and ordered predecessor contracts named by the route. |
| `GATE-SET-CROSS-FAMILY` | gate | Set relations and algebra spanning sparse and ordered stores. | After the exact sparse and ordered predecessor contracts named by the route. |
| `GATE-BULK-CONSTRUCTION-CROSS-FAMILY` | gate | Extend/collect behavior whose concrete implementations span several topologies. | After every topology predecessor named by the exact implementation route. |
| `GATE-INDEX-CROSS-FAMILY` | gate | Indexing behavior spanning dense, deque, and keyed topologies. | After every topology predecessor named by the exact implementation route. |
| `GATE-CONVERSION-CROSS-FAMILY` | gate | Cross-topology construction or conversion behavior. | After every topology predecessor named by the exact implementation route. |
| `GATE-LINKED-COMPOSITION` | gate | Linked-list composition after dense, identity, and recursive prerequisites are made exact. | After every predecessor named by the route; it is not a substitute for any predecessor. |
| `GATE-FAMILY-ALLOCATION-ERROR` | gate | Allocation-error evidence delegated to each operation family that exposes recoverable allocation. | At every exact delegated owner outcome, with `DIM-FAILURE` rebound locally. |
| `GATE-RAW-SPELLING-REJECTION` | gate | Preserves rejection of writer-visible raw or uninitialized spelling while retaining the underlying need for a checked family route. | Boundary evidence only; rejection never closes the underlying need. |
| `GATE-ROPE-POST-DENSE-ORDERED` | gate | Optional unique rope composition. | Only after `F-DENSE` and `F-ORDERED`; it never blocks unless promoted or required. |

## 4. Local proof dimensions

| ID | Kind | Exact local obligation |
|---|---|---|
| `DIM-ACCESS` | dimension | Provenance, reborrow, disjointness, invalidation, and result-borrow access relations. |
| `DIM-OWNERSHIP` | dimension | Initialization, move-out, replacement, swap, relocation, clone, and exact destruction relations. |
| `DIM-FAILURE` | dimension | Normal exit, abandonment, abort, capacity failure, allocation failure, callback failure, and failure atomicity. |
| `DIM-BEHAVIOR` | dimension | Callable behavior effects, result provenance, law containment, and direct static dispatch requirements. |
| `DIM-STORED-BORROW` | dimension | Stored or returned borrow-bearing payload scope and its exact complement branch. |
| `DIM-RESOURCE-LIFETIME` | dimension | Acquisition, retained owner, release, deliberate leak boundary, and resource-lifetime accounting. |

Every dimension is rebound at exact `member_contract_id` and `outcome_id`
granularity. A cluster-level union is not a proof for any child.

### 4.1 Exact concrete-implementation topology classes

This compact table is the human authority for the closed exact implementer
classifier. Exact implementer membership remains in the generated 334-row TSV;
it is not duplicated here.

<!-- G0_TOPOLOGY_CLASS_AUTHORITY_BEGIN -->
| Class ID | Primary refinement family or gate | Required predecessor families | Required predecessor gates | Implicated/reopening families | Implicated/reopening gates | Semantic rationale |
|---|---|---|---|---|---|---|
| `CLASS-DENSE` | `F-DENSE` | `NONE` | `NONE` | `F-DENSE` | `NONE` | Contiguous Vec, slice, array, and boxed-slice or boxed-array storage is refined by the dense family. |
| `CLASS-RECURSIVE-BOX` | `F-RECURSIVE` | `NONE` | `NONE` | `F-RECURSIVE` | `NONE` | Generic Box ownership is the uniquely owned recursive substrate; an allocator type parameter alone imports no allocator-service contract. |
| `CLASS-TEXT` | `F-TEXT` | `F-DENSE` | `NONE` | `F-TEXT` | `NONE` | String, str, and boxed str require the text family, with dense storage retained only as a predecessor. |
| `CLASS-DEQUE` | `F-DEQUE` | `NONE` | `NONE` | `F-DEQUE` | `NONE` | VecDeque implementers retain the distinct ring and rebalance topology. |
| `CLASS-SPARSE` | `F-SPARSE` | `F-DENSE` | `NONE` | `F-SPARSE` | `NONE` | Hash-map and hash-set implementers require sparse occupancy, with dense storage retained only as a predecessor. |
| `CLASS-ORDERED` | `F-ORDERED` | `F-DENSE,F-IDENTITY,F-RECURSIVE` | `NONE` | `F-ORDERED` | `NONE` | B-tree map and set implementers retain comparison-ordered topology after reviewed dense node arrays and both general recursive-owner and stable-identity node routes; candidates need not combine them or pay unused metadata. |
| `CLASS-HEAP` | `F-HEAP` | `F-DENSE` | `NONE` | `F-HEAP` | `NONE` | BinaryHeap implementers require heap repair semantics, with dense storage retained only as a predecessor. |
| `CLASS-LINKED` | `GATE-LINKED-COMPOSITION` | `F-DENSE,F-IDENTITY,F-RECURSIVE` | `NONE` | `NONE` | `GATE-LINKED-COMPOSITION` | LinkedList implementers pass through the linked-composition gate after dense, identity, and recursive predecessors. |
| `CLASS-SHARED` | `F-SHARED` | `NONE` | `NONE` | `F-SHARED` | `NONE` | Generic Rc and Weak implementers are owned by shared-ownership refinement and do not import dense sequence topology. |
| `CLASS-SHARED-DENSE` | `F-SHARED` | `F-DENSE` | `NONE` | `F-SHARED` | `NONE` | Rc slice and array wrappers are owned by shared refinement while dense payload storage remains a predecessor; dense cannot disposition the wrapper child. |
| `CLASS-SHARED-TEXT` | `F-SHARED` | `F-TEXT` | `NONE` | `F-SHARED` | `NONE` | Rc<str> is owned by shared refinement while sealed text is a predecessor; text cannot disposition the wrapper child. |
| `CLASS-DYNAMIC-BORROW` | `F-DYNAMIC-BORROW` | `NONE` | `NONE` | `F-DYNAMIC-BORROW` | `NONE` | RefCell and guard implementers require runtime dynamic-borrow topology and do not import dense sequence topology. |
| `CLASS-ITERATION` | `F-ITERATION` | `NONE` | `NONE` | `F-ITERATION` | `NONE` | Step implementers are exact scalar or address traversal sources owned by iteration refinement. |
<!-- G0_TOPOLOGY_CLASS_AUTHORITY_END -->

### 4.2 Exact concrete-implementation operation-gate assignments

The following four-row authority is the complete positive assignment from an
exact concrete implementation relation's owning cluster to an additional
operation gate. Every concrete implementation relation whose owning cluster is
absent from this table has the exact closed-negative assignment `NONE` / `NONE`.
The child-specific immediate predecessor is additive: it binds that exact child
to its topology primary but does not replace or reduce the operation gate's
complete cluster-route predecessor family and gate sets.

<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_BEGIN -->
| Owning cluster ID | Additional operation gate stage | Child-specific immediate predecessor policy | Expected concrete implementation relation count | Semantic rationale |
|---|---|---|---|---|
| `TRAIT-EXTEND-01` | `GATE-BULK-CONSTRUCTION-CROSS-FAMILY` | `EXACT_TOPOLOGY_PRIMARY` | `22` | Extend semantics remain an independently applicable bulk-construction target after the exact implementer topology primary. |
| `TRAIT-COLLECT-01` | `GATE-BULK-CONSTRUCTION-CROSS-FAMILY` | `EXACT_TOPOLOGY_PRIMARY` | `21` | Collect semantics remain an independently applicable bulk-construction target after the exact implementer topology primary. |
| `TRAIT-INDEX-01` | `GATE-INDEX-CROSS-FAMILY` | `EXACT_TOPOLOGY_PRIMARY` | `14` | Index semantics remain an independently applicable cross-family target after the exact implementer topology primary. |
| `TRAIT-CONVERT-01` | `GATE-CONVERSION-CROSS-FAMILY` | `EXACT_TOPOLOGY_PRIMARY` | `40` | Conversion semantics remain an independently applicable cross-family target after the exact implementer topology primary. |
<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_END -->

### 4.3 Coarse cluster route-group decisions

This compact table authorizes route categories and their dependency semantics.
Exact membership of all 276 clusters remains in the generated routing TSV and
its independently pinned verifier.

<!-- G0_CLUSTER_ROUTE_GROUP_AUTHORITY_BEGIN -->
| Route group ID | Primary refinement family or gate | Required predecessor families | Required predecessor gates | Explicit implicated/reopening families | Required dimensions | Route state | Cross-topology or reopening policy | Rationale and source |
|---|---|---|---|---|---|---|---|---|
| `ROUTE-ACTIVE-FAMILY-DIRECT` | `EXACT_FAMILY_BY_ASSIGNMENT` | `EXACT_ROUTE_LOCAL_SET_OR_NONE` | `NONE` | `PRIMARY_FAMILY_ONLY` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `ACTIVE` | `INDEPENDENT_EXACT_CHILD_REBIND;PREDECESSORS_NEVER_GAIN_APPLICABILITY` | Direct active family route; G0 cluster-routing discipline. |
| `ROUTE-ACTIVE-FAMILY-COMPOUND` | `EXACT_FAMILY_BY_ASSIGNMENT` | `EXACT_ROUTE_LOCAL_SET_OR_NONE` | `NONE` | `PRIMARY_PLUS_EXPLICIT_REOPENING_SET` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `ACTIVE` | `DISCOVERY_UNION_ONLY;EVERY_EXACT_CHILD_SPLIT_AND_REBOUND` | Compound active family route; boxed-init hostile review. |
| `ROUTE-ACTIVE-TRAIT-FAMILY` | `COARSE_START_FAMILY` | `EXACT_ROUTE_LOCAL_SET_OR_NONE` | `NONE` | `EXACT_IMPL_TOPOLOGY_DISCOVERY_UNION` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `ACTIVE` | `EXACT_IMPL_KEY_TOPOLOGY_PRIMARY_OVERRIDES_COARSE_FAMILY` | Trait-family route; exact 334-row topology routing. |
| `ROUTE-ACTIVE-TRAIT-GATE` | `EXACT_OPERATION_GATE` | `EXACT_IMPL_TOPOLOGY_FAMILY_UNION` | `EXACT_DISTINCT_TOPOLOGY_GATE_UNION_OR_NONE` | `EXACT_IMPL_TOPOLOGY_DISCOVERY_UNION` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `ACTIVE` | `OPERATION_GATE_ADDITIONAL;EVERY_GATE_TERMINAL_INDEPENDENT` | Trait operation-gate route; gate-composition hostile review. |
| `ROUTE-ACTIVE-CROSS-FAMILY-GATE` | `EXACT_GATE_BY_ASSIGNMENT` | `EXACT_ROUTE_LOCAL_SET_OR_NONE` | `NONE` | `EXACT_REOPENING_SET_OR_NONE` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `ACTIVE` | `GATE_AFTER_EXACT_PREDECESSORS;NO_PREDECESSOR_EVIDENCE_OWNERSHIP` | Cross-family gate route; typed gate vocabulary. |
| `ROUTE-SCOPED-LATER-FAMILY` | `EXACT_LATER_FAMILY` | `EXACT_TRUE_TOPOLOGY_PREDECESSORS_OR_NONE` | `NONE` | `PRIMARY_FAMILY_ONLY` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `SCOPED_LATER` | `NO_DRAFT_OR_IMPLEMENTATION_WITHOUT_OWNER_REAUTHORIZATION` | Later-family preservation; G0 owner scope ruling. |
| `ROUTE-BOUNDARY-FAMILY` | `EXACT_BOUNDARY_FAMILY` | `EXACT_CHECKED_INPUT_FAMILIES_OR_NONE` | `NONE` | `PRIMARY_FAMILY_ONLY` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `BOUNDARY` | `CHECKED_NEED_ONLY;NO_RUST_RAW_SPELLING_IMPORT` | Boundary-family route; G0 boundary evidence review. |
| `ROUTE-BOUNDARY-REJECTION-GATE` | `GATE-RAW-SPELLING-REJECTION` | `EXACT_CHECKED_INPUT_FAMILIES_OR_NONE` | `NONE` | `EXACT_CHECKED_REOPENING_SET_OR_NONE` | `EXACT_LOCAL_DIMENSION_SET_OR_NONE` | `BOUNDARY` | `RAW_OR_UNINITIALIZED_SPELLING_REJECTED;CHECKED_NEED_PRESERVED` | Boundary rejection route; CONSTITUTION and PATTERNS P8. |
| `ROUTE-DELEGATED-GATE` | `GATE-FAMILY-ALLOCATION-ERROR` | `NONE` | `NONE` | `NONE` | `DIM-FAILURE` | `DELEGATED` | `EXACT_DELEGATED_OWNER_OUTCOMES_ONLY;NO_INDEPENDENT_MEMBER` | Delegated allocation route; ALLOC-ERROR-01 hostile review. |
<!-- G0_CLUSTER_ROUTE_GROUP_AUTHORITY_END -->

### 4.4 Cluster-assignment semantic rationales

Every cluster route row carries one closed rationale ID from this table. The
structural category governs mechanics; this table explains the non-obvious
semantic reason for its primary, predecessor, and reopening assignment.

<!-- G0_CLUSTER_ASSIGNMENT_RATIONALE_AUTHORITY_BEGIN -->
| Rationale ID | Assignment rule | Semantic rationale | Source authority |
|---|---|---|---|
| `ASSIGN-OUTER-TOPOLOGY-DIRECT` | Use the cluster's externally observable storage or traversal topology as its primary family; do not infer another topology from capability labels. | Distinct deque, ordered, sparse, iteration, and other outer contracts remain independently refinable, and predecessors never gain evidence applicability. | G0-CORE-REPORT.md Sections 1 and 9; G0-COVERAGE-CLUSTER-REGISTRY.tsv |
| `ASSIGN-DENSE-FOUNDATION` | Route contiguous views, initialized-prefix sequence operations, and generic replace or take starting points to F-DENSE with local dimensions rebound per exact child. | These operations require contiguous live-range state or dense reusable ownership facts; the coarse cluster remains discovery rather than closure. | general-purpose-data-structure-capability-RESEARCH.md Sections 4 through 6; RUST-DATA-CONTRACT-CENSUS.tsv |
| `ASSIGN-GENERIC-BOX` | Route generic Box construction to F-RECURSIVE without importing allocator-service or dense-sequence topology. | Generic Box is the uniquely owned recursive substrate; an allocator type parameter alone is not a user-visible allocator contract. | G0-CORE-REPORT.md later-family boundary; RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv |
| `ASSIGN-BOXED-INIT-SPLIT` | Start BOX-INIT-01 in F-RECURSIVE and explicitly reopen F-DENSE for boxed slice or array children, requiring independent child splitting. | Scalar generic Box and boxed dense payload constructors share a coarse census cluster but not one closure family. | RUST-DATA-CONTRACT-CENSUS.tsv BOX-INIT-01; hostile topology review |
| `ASSIGN-SEQUENCE-BACKED-HEAP` | Route BinaryHeap operations to F-HEAP after the exact dense backing-sequence predecessor. | Heap repair is the outer contract while dense storage is reusable input, not a second owner of heap evidence. | general-purpose-data-structure-capability-RESEARCH.md Sections 4 and 5; canonical predecessor union |
| `ASSIGN-TEXT-OVER-DENSE` | Route byte and UTF-8 string semantics to F-TEXT after F-DENSE storage. | Text validity and boundary-safe edits are the outer contract; dense bytes are a predecessor and cannot disposition text evidence. | general-purpose-data-structure-capability-RESEARCH.md Section 4; G0-CORE-REPORT.md |
| `ASSIGN-LINKED-COMPOSITION` | Route linked-list operations through GATE-LINKED-COMPOSITION after dense, identity, and recursive predecessors. | No one predecessor alone supplies linked composition, stable node identity, recursive ownership, and bounded destruction. | general-purpose-data-structure-capability-RESEARCH.md Sections 4 and 7.1; gate vocabulary |
| `ASSIGN-EXACT-TRAIT-TOPOLOGY` | Use the coarse trait family only as a starting stage and replace it per concrete impl_key with the exact topology primary. | A trait cluster spans unrelated implementer topologies; flat family custody would let one family erase another wrapper or container obligation. | RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv; G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv |
| `ASSIGN-OPERATION-GATE-COMPOSITION` | Keep the cross-family operation gate additionally applicable after every exact topology primary and distinct topology gate predecessor. | Operation semantics and implementer topology are separate obligations keyed by the same evidence child; neither terminal or exclusion erases the other. | gate vocabulary; exact trait-impl topology routing; hostile gate-composition review |
| `ASSIGN-SCOPED-LATER-WRAPPER` | Preserve the exact later wrapper family and only its true topology predecessors, with all ownership, access, failure, and resource dimensions rebound locally. | Pinning, shared ownership, dynamic borrow, Unicode, and type identity are distinct outer contracts and remain unauthorized for implementation. | G0-CORE-REPORT.md later-family boundary; owner scope ruling |
| `ASSIGN-RAW-BOUNDARY` | Retain checked ABI, allocation, or underlying family needs while rejecting writer-visible raw or uninitialized spelling. | Boundary evidence preserves completeness without importing Rust unsafe mechanisms or weakening source-level checks. | CONSTITUTION.md; PATTERNS.md P8; G0-CORE-REPORT.md boundary evidence |
| `ASSIGN-DELEGATED-ALLOCATION` | Delegate recoverable allocation-error evidence to every exact delegated owner outcome with DIM-FAILURE rebound locally. | The row has no independent operation and cannot close until all named owner outcomes discharge their branch. | RUST-DATA-CONTRACT-CENSUS.tsv ALLOC-ERROR-01; gate vocabulary |
<!-- G0_CLUSTER_ASSIGNMENT_RATIONALE_AUTHORITY_END -->

## 5. Route states

| ID | Meaning |
|---|---|
| `ACTIVE` | The demand is in the current sequential unique-owner research boundary. This is accounting, not authorization. |
| `SCOPED_LATER` | The demand is preserved for a named later family and requires owner authorization before its lock is drafted. |
| `BOUNDARY` | The Rust spelling is inadmissible or frame-owned; the underlying checked need remains accounted. |
| `PROTECTED` | Reserved for protected controls in the role registry. Coarse coverage clusters may not use this state. |
| `DELEGATED` | The row has no independent operation and is discharged only through every exact named owner outcome. |

The canonical state set is exactly `ACTIVE`, `SCOPED_LATER`, `BOUNDARY`,
`PROTECTED`, and `DELEGATED`. There is no unknown, inferred, or default state.

## 6. Requirement-assignment authority

The following table is the human-readable assignment authority consumed by the
family-requirement builder. A row's owner or gate has closure custody; its
predecessors are exact prerequisites; its implicated set controls reopening and
per-topology rebinding. Implication never lets one family dispose of another
family's exact evidence child.

`F-IDENTITY` is staged after `F-SPARSE` because recyclable pools and W-POOL
must exercise the same public checked arbitrary-occupancy contract as H-STORE;
inventing a second private occupancy substrate would duplicate a required
capability and evade the ordinary-library witness. This dependency imports no
finished map, set, pool, or container. It imports only the exact public sparse
state contract proven by the predecessor. Dense graph or arena dependencies
remain separate row-local imports.

The generated registry separates owner-lock dispositions from implicated-family
rebind dispositions. A predecessor is an input to an owner-local M/W/H
obligation; it cannot prove that obligation. Owner closure therefore permits
only `REQUIRED_IN_LOCK` or claim-blocking exclusion, with the role-specific
protected and optional cases. Only a row marked
`EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY` creates a role terminal in an
implicated lock. That terminal is the indivisible
`PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED`: it proves both the closure owner's
exact reusable unit and the implicated family's topology-local member, outcome,
and canary units. It is valid even when the source row has no earlier
`required_predecessor_family_ids`, because the source row's closure owner is the
reuse authority. Other implicated sets control reopening and claim boundaries
only and use `NOT_APPLICABLE_REOPENING_ONLY` rather than creating a second
closure obligation.

<!-- G0_REQUIREMENT_ASSIGNMENTS_BEGIN -->
| Requirement subject | Owner or gate | Required predecessors | Implicated families | Rebind policy |
|---|---|---|---|---|
| Fixed buffer of Copy scalars | ALL-FAMILY-LOCKS | NONE | ALL-FAMILY-LOCKS | EVERY_CANDIDATE_EXECUTES |
| Fixed AoS record buffer | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Unknown-length append | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Append affine value | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Grow/shrink contiguous sequence | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Pop affine value | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Ordered insert/remove and unordered `swap_remove` | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Swap two dynamic elements | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Clear/truncate | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Deep clone and bulk move-append | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Stable retain and eager drain/splice | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Lazy drain cursor | F-DENSE | NONE | F-DENSE | OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN |
| Generic unstable and stable sort | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Stack adapter | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| FIFO queue/deque | F-DEQUE | NONE | F-DEQUE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Priority queue | F-HEAP | F-DENSE | F-HEAP | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Hash map/set | F-SPARSE | NONE | F-SPARSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Ordered map/set | F-ORDERED | F-DENSE,F-IDENTITY,F-RECURSIVE | F-ORDERED | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Append-only AST/DAG/graph | ALL-FAMILY-LOCKS | NONE | ALL-FAMILY-LOCKS | EVERY_CANDIDATE_EXECUTES |
| Recyclable stable pool | F-IDENTITY | F-SPARSE | F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Frozen graph | F-IDENTITY | F-DENSE | F-DENSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Dynamic graph with deletion | F-IDENTITY | F-DENSE | F-DENSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Singly owned recursive list | F-RECURSIVE | NONE | F-RECURSIVE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Doubly linked/cyclic list | F-IDENTITY | F-DENSE | F-DENSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Homogeneous bump arena/slab | F-ARENA | F-DENSE | F-DENSE,F-ARENA | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Inline-to-heap small sequence | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Unseen storage-bearing structure | F-SPARSE | F-DENSE | F-SPARSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| LRU cache | F-IDENTITY | F-SPARSE | F-SPARSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Indexed priority queue | F-HEAP | F-DENSE,F-SPARSE | F-DENSE,F-SPARSE,F-HEAP | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Bytes and UTF-8 text builder | F-TEXT | F-DENSE | F-DENSE,F-TEXT | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| Borrowed, uniq, and owning iteration | F-DENSE | NONE | F-DENSE,F-DEQUE,F-SPARSE,F-ORDERED,F-HEAP,F-IDENTITY,F-RECURSIVE,F-ARENA,F-TEXT | EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY |
| B-FIX | ALL-FAMILY-LOCKS | NONE | ALL-FAMILY-LOCKS | EVERY_CANDIDATE_EXECUTES |
| B-P2 | ALL-FAMILY-LOCKS | NONE | ALL-FAMILY-LOCKS | EVERY_CANDIDATE_EXECUTES |
| W-POOL | F-IDENTITY | F-SPARSE | F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-ARENA | F-ARENA | F-DENSE | F-DENSE,F-ARENA | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-SMALL | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-RECUR | F-RECURSIVE | NONE | F-RECURSIVE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-GRAPH | F-IDENTITY | F-DENSE | F-DENSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-ECS | F-IDENTITY | F-DENSE | F-DENSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-GAP | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| W-PIPE | F-ITERATION | F-DENSE | F-DENSE,F-DEQUE,F-SPARSE,F-ORDERED,F-HEAP,F-IDENTITY,F-ITERATION,F-RECURSIVE,F-ARENA,F-TEXT | EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY |
| H-FLATSET | F-DENSE | NONE | F-DENSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| H-STORE | F-SPARSE | F-DENSE | F-SPARSE | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| H-LRU | F-IDENTITY | F-SPARSE | F-SPARSE,F-IDENTITY | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| H-IPQ | F-HEAP | F-DENSE,F-SPARSE | F-DENSE,F-SPARSE,F-HEAP | OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN |
| O-SLAB | F-IDENTITY | NONE | F-IDENTITY | OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN |
| O-ROPE-UNIQUE | GATE-ROPE-POST-DENSE-ORDERED | F-DENSE,F-ORDERED | F-DENSE,F-ORDERED | OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN |
| O-INTRUSIVE | F-PIN-ADDRESS | NONE | F-IDENTITY,F-PIN-ADDRESS | OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN |
| O-LAZY-DRAIN | F-DENSE | NONE | F-DENSE | OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN |
<!-- G0_REQUIREMENT_ASSIGNMENTS_END -->

## 7. Canonical mandatory predecessor unions

A Family Lock's canonical required predecessor set is the union of the
row-local predecessor sets for all applicable non-O rows it owns. An O row
joins only after owner promotion or when a mandatory contract requires it.
This rule preserves exact row-local evidence: for example, the base hash-map
operation has no dense predecessor, while H-STORE adds dense to the canonical
sparse-lock union; dense does not thereby become an implicated family for the
H-STORE row.

| Closure owner or control scope | Canonical mandatory predecessors | Derivation |
|---|---|---|
| `ALL-FAMILY-LOCKS` | NONE | Protected controls execute directly in every lock. |
| `F-DENSE` | NONE | Current dense M/W/H rows have no earlier family input. |
| `F-DEQUE` | NONE | Deque is a distinct topology; no current mandatory role imports another family. |
| `F-ORDERED` | `F-DENSE`,`F-IDENTITY`,`F-RECURSIVE` | D11 places ordered storage after G3: dense supplies affine node arrays and the two general node routes remain available without authorizing an ordered-only raw substrate. Candidates need not combine them or pay unused identity metadata. |
| `F-RECURSIVE` | NONE | Generic box/recursive ownership is first frozen in this family. |
| `F-PIN-ADDRESS` | NONE | The only current owner row is optional and unpromoted. |
| `F-ARENA` | `F-DENSE` | The arena operation and W-ARENA import the exact dense backing-owner contract. |
| `F-SPARSE` | `F-DENSE` | H-STORE imports exact public dense capabilities while directly exercising sparse state. |
| `F-TEXT` | `F-DENSE` | The text builder imports the selected dense byte-storage contract. |
| `F-ITERATION` | `F-DENSE` | W-PIPE reuses the frozen dense traversal contract and adds topology-local composition. |
| `F-HEAP` | `F-DENSE`,`F-SPARSE` | Priority queue imports dense sequence storage; H-IPQ additionally imports sparse keyed positions. |
| `F-IDENTITY` | `F-DENSE`,`F-SPARSE` | Graph/ECS rows import dense storage; recyclable pool/W-POOL and H-LRU import public sparse state. |
| `GATE-ROPE-POST-DENSE-ORDERED` | `F-DENSE`,`F-ORDERED` only if promoted | O-ROPE-UNIQUE is optional and currently creates no mandatory gate. |

The graph is acyclic. A row-local predecessor may be a strict subset of its
owner's canonical union, and a predecessor is never copied into the row's
implicated/reopening set merely because it is an input.

## 8. Machine-enforced invariants

The builders and verifiers must reject all of the following:

1. an undocumented family, gate, dimension, state, or control scope;
2. a `DIM-*` token used as owner, gate, predecessor, or closed family;
3. a gate used without its exact predecessor set;
4. a protected baseline that does not run against every candidate;
5. a predecessor disposition used to discharge an owner-local requirement, or
   a cross-topology reuse/local-rebind terminal split into alternatives;
6. a coarse cluster used as a member contract, outcome contract, capability or
   cost inheritance source, family `E`/`P` unit, candidate, or scored unit;
7. an implementation child classified by fuzzy matching, an unknown fallback,
   or a self-selected topology subset;
8. an exclusion in one family erasing another applicable family's obligation;
9. a selector disposition before independently exhaustive child expansion; or
10. a changed assignment, source, or typed vocabulary without regenerated bytes,
   focused verification, and a new exact-hash hostile review.

The route-group and exact implementation-child assignments live in the
generated routing registries and their closed builders. Those builders validate
every token and dependency against this document, and their verifiers pin exact
row sets and digests. The generated artifacts are evidence routing only; they
do not select exact member semantics or a production mechanism.
