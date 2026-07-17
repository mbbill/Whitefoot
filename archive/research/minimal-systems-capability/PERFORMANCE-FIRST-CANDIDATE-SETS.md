# Performance-First Candidate Capability Sets

Date: 2026-07-15

Status: paper candidates for comparison; no candidate is selected, proved, measured, authorized for validation, or approved for production.

## 1. Construction discipline

These candidates are derived from the 15 conjunctive demand families in `PERFORMANCE-EXPRESSIVENESS-GAP-LEDGER.tsv`. They are not translations of the previous P1-P9/Q1-Q6 basis, a Rust API list, or a privilege-boundary design.

Each candidate must provide a route for every gap. A route may be:

- a general language, type, ownership, checker, or proof rule;
- an ordinary checked-library construction;
- a fixed builtin or runtime/target leaf for behavior ordinary source cannot create;
- an explicit exclusion from the intended systems workload.

No candidate below takes the exclusion route. Each therefore includes the same exact-machine lower bound and differs in how ordinary checked code represents storage state, ownership transitions, access authority, cleanup, and optimizer facts.

The analytical names below are not proposed syntax. A later syntax decision is separately gated.

The extension boundary is the material difference among the candidates. Candidate A admits a new ordinary-library protocol when its proof reduces to the fixed logic. Candidate B admits a new library data structure only by composing the closed orthogonal topology/place/footprint forms; its algorithms remain ordinary functions. Candidate C admits only the operations already attached to compiler-known families; adding a new low-level family or protocol is a language change.

## 2. Common lower bound: C0

The gap ledger identifies capabilities that cannot be derived merely by changing the representation of an ordinary value. All three candidates include C0. A candidate may implement a C0 item directly in the compiler, through an exact runtime body, or through a target provider selected by the compiler, but ordinary source cannot invent its machine semantics.

| ID | Exact capability | Combination rule | Removal witness |
|---|---|---|---|
| C0-1 | Opaque generative modules. A module may hide representation constructors and fields; each generative owner type has a distinct static identity. Public operations remain ordinary checked functions. Sealing erases and grants no optimizer fact by itself. | Hidden state can be consumed by any candidate-specific transition rule. A fact is usable only when separately produced by a checked rule. | Remove it and PF-G05 requires forgeable length/occupancy state, runtime access control, or compiler-only named containers. |
| C0-2 | Stateful generic callable contracts. A callable declares capture ownership, per-call effects, invocation cardinality, pre/post leaf disposition, and per-result provenance. Concrete calls are monomorphized and direct. | Callable state participates in the same owner and cleanup accounting as other fields. Laws become facts only through C0-5. | Remove it and W-PIPE, clone-from, comparison, hashing, callback failure, and owning traversal require dynamic dispatch, materialization, or named special cases. |
| C0-3 | Selective immovability and suspension. An owner may enter an immovable state tied to one physical root; only declared projections and retained operations may borrow it. Cancellation and destruction close the retained state. Ordinary owners remain movable. | It composes with candidate-specific live-state rules and C0-7 allocation roots; logical identity never implies physical immovability. | Remove it and PF-G11 requires universal heap indirection, copying state machines, unchecked addresses, or workload exclusion. |
| C0-4 | Refinement validation. A checked deterministic predicate may seal an exact value/root/version as satisfying a declared refinement; every listed mutation invalidates it. Foreign values must validate before sealing. | The refinement proposition is consumed by C0-5; it never changes storage liveness or ownership by itself. | Remove it and PF-G08 requires repeated validation, copied storage, permanent validity tags, or trust. |
| C0-5 | Fixed verified-fact interface. The compiler owns a closed fact-kind table. Each fact records proposition, root, region/version, producer, consumers, invalidators, transfer rule, and facts-off behavior. Producers are deterministic checker derivations, explicit runtime checks, or artifacts accepted by a fixed verifier; source assertions never create assumptions. | Candidate-specific state and footprint rules may produce entries already present in the table. Adding a fact kind is a language change. Explicit checks survive; unsupported proof retains the check. | Remove it and PF-G15 retains demonstrated bounds, alias, algebraic, refinement, state, and code-shape costs. Make it writer-extensible and soundness or optimizer correctness becomes forgeable. |
| C0-6 | Recoverable effect and commit algebra. Signatures distinguish success, owner-returning precommit failure, failure after documented partial progress, and abort. Every normal result states exact owner and cleanup disposition. | Candidate-specific transitions must identify their commit point. C0-2 callback outcomes and C0-7 allocation outcomes use the same algebra. | Remove it and PF-G07 loses offered owners, copies state for rollback, conflates abort and recovery, or cannot express try-style operations. |
| C0-7 | Exact allocation-root table. Fixed rows create, resize, and release physical roots with checked size/alignment, recoverable or aborting shortage policy, and exact ownership transfer. No row accepts an arbitrary allocator contract from source. | Candidate storage owners hold these roots. Reallocation must use the candidate's relocation rule; fixed storage does not acquire capacity metadata. | Remove it and growable, inline-spill, recursive, and recoverable-allocation witnesses cannot be implemented without a trusted external allocator or changed failure contract. |
| C0-8 | Thread and atomic event table plus one language memory model. Rows cover thread creation/join, atomic load/store/read-modify-write/fence, blocking/wakeup, and the ownership transfer and happens-before relations attached to each. Each target maps a row to exact supported events or rejects it. | Shared-lifecycle libraries are ordinary checked code over candidate ownership state plus these events. Unique-owner code emits no atomic or synchronization event. | Remove it and PF-G10 requires external trust, coarse serialization, race-prone source, or exclusion of concurrency. Leave the memory model unspecified and no safety proof is meaningful. |
| C0-9 | Exact external-event tables. Each platform profile enumerates I/O, handle/resource, clock/randomness, process, dynamic-provider, and FFI-frame rows with buffer footprints, partial progress, blocking/readiness, error, ownership transfer, close, and foreign-validation semantics. There is no generic syscall, arbitrary symbol contract, or writer-defined ABI descriptor. | Rows consume candidate borrows and owners, return C0-6 outcomes, and may produce only C0-5 fact kinds. A profile lacking a row rejects the program. | Remove it and PF-G12 requires staging copies, duplicated calls, leaked handles, unchecked FFI, or workload exclusion. |
| C0-10 | Exact target/device-event tables. Each target profile enumerates scalar special operations, SIMD, feature queries, MMIO, DMA, device memory, cache, ordering, reset, and fallback rows with totality, alignment, footprint, lifetime, and machine-event contracts. There is no arbitrary opcode or semantic descriptor. | Rows consume candidate places/borrows and C0-3 address-stable owners where required. Portable fallback is an ordinary checked function selected explicitly. | Remove it and PF-G13 requires scalarization, helper calls, extra memory traffic, unchecked devices, or workload exclusion. |
| C0-11 | Executable-image lifecycle table. Fixed rows cover writable allocation, relocation application, provider binding, permission transition, instruction-cache coherence, immutable-image identity, activation, quiescence, and release. Write and execute authority never coexist. | Any fact about generated code binds to the final immutable loaded-image identity through C0-5. Candidate ownership accounts for code and data roots. | Remove it and PF-G14 requires an interpreter, writable executable memory, unchecked loading, permanent indirection, or workload exclusion. |

### C0 limitations

C0-8 through C0-11 specify the required table semantics but do not enumerate platform rows. That enumeration remains a finite target-profile obligation before any candidate can claim production completeness. The three candidates are comparable at the language-capability level because they share this lower bound; none receives evidence credit for an unenumerated row.

### Universal acceptance invariants

No candidate survives comparison unless all of these hold:

1. A payload read or borrow is accepted only with machine-verified evidence that the exact place contains a live T.
2. Every affine owner is in exactly one live place, result, argument, protocol state, or destruction event; storage relocation never implies Clone.
3. Every normal or recoverable exit leaves each reachable public owner valid and accounts for every offered owner. Trap-to-abort owes no cleanup but cannot perform an invalid read, duplicate drop, race, or stale-fact access first.
4. Every borrow leaf retains its exact physical root, place, region, and invalidators across aggregate formation, return, storage, relocation, and protocol movement.
5. Shared access is accepted only through static uniqueness or the selected memory-model and synchronization events; data races remain unrepresentable.
6. An optimizer fact is usable only from a fixed machine-verified producer, is invalidated on every relevant mutation or interference, and changes no source semantics when disabled.
7. No writer-accessible operation disables a check, asserts unverified topology, invents a machine event, installs a proof rule, or describes an arbitrary syscall, opcode, ABI, or optimizer fact.
8. A stronger representation never taxes a weaker protected shape unless the weaker program explicitly selects it.

## 3. Candidate A: proof-indexed resource calculus

### 3.1 Thesis

Represent resource state, liveness, ownership, provenance, identity, and interference as explicit static indices and machine-verified propositions. Ordinary libraries may define new resource protocols without new compiler-known topology kinds, provided all proofs reduce to one fixed logic and verifier.

This candidate maximizes semantic generality and minimizes the number of topology-specific language forms. Its risk is that the proof and checker system becomes a second programming language or exceeds normal-frontend effort.

### 3.2 Exact capability set

| ID | Capability and semantics | Why selected; deletion consequence |
|---|---|---|
| A-1 | Indexed resource types `R<S, V>`, analytically: `R` is a generative resource identity, `S` is an abstract state term, and `V` is a monotonically changing version. Moving the owner preserves all three; a state transition consumes the old indexed owner and returns a new one. State and version erase unless the representation contract explicitly stores metadata. | It is the common carrier for partial live sets, identity, shared lifecycle, and suspended state. Delete it and those families require separate built-in state machines or runtime tags. |
| A-2 | Fixed resource logic. The only propositions are equality/inequality, integer ranges and checked arithmetic, finite sets and maps over places, disjointness, finite products/sums, reachability through declared owner edges, per-leaf provenance, state transition, version equality, and happens-before edges from C0-8. Quantification is limited to structurally bounded ranges owned by one resource. User code cannot add axioms or proof rules. | This is sufficient in principle to state Full, Prefix, Ring, Sparse, Dependent, Hole, refinement, identity, footprint, and lifecycle relations. Remove any listed proposition class and the corresponding ledger family becomes a compiler special case; admit arbitrary predicates and decidability and trust boundaries collapse. |
| A-3 | Proof terms and proof-carrying function signatures. A function may require and return erased proofs over declared parameters and results. Proof search is untrusted; only a deterministic kernel accepts a term. Proof obligations that cannot be discharged are rejected or retain an explicitly specified runtime check. | This lets ordinary libraries preserve invariants across calls without hidden runtime descriptors. Delete it and indexed states cannot be composed modularly; trust proofs and facts become forgeable. |
| A-4 | Linear place algebra. From a resource proof, code may derive affine `Live<R,p,T,V>` or `Vacant<R,p,T,V>` place authority, bounded range authority, or a finite disjoint product. Authority is non-copyable; a live read/borrow/move/drop or vacant initialization consumes or reborrows exactly the relevant authority. | It connects logical propositions to memory access. Delete it and A-2 proves abstract state but cannot safely authorize payload operations; encode it at runtime and PF-G02/G03 acquire tags or tables. |
| A-5 | General transition contracts. A transition signature lists pre-state, post-state, success/failure states, commit point, exact invocation counts, per-owner disposition, and facts produced or invalidated. The body is checked against this relation. | It covers relocation, replacement, sparse changes, ECS migration, shared lifecycle, and callback outcomes without named operations. Delete it and those transitions become builtins or cannot cross function boundaries. |
| A-6 | Exact-use or derived-cleanup protocols. A transition value is either abandonable because it owns a valid base state, exact-use on every normal path, or equipped with checker-derived cleanup whose post-state is part of A-5. Trap edges owe no cleanup but cannot read invalid storage before abort. | It closes holes, drains, entry guards, suspension, and partial construction. Delete it and affinity alone permits stranded invalid states. |
| A-7 | Provenance and footprint propositions. Every borrow leaf names its physical root and region; a protocol value may carry a finite product or sum. Unique access requires a proof that its footprint is disjoint from all live incompatible footprints. Dynamic checks may mint a version-bounded proof once. | It serves returned/stored borrows, get-many-mut, projected guards, and alias facts. Delete it and PF-G04 needs copying, runtime borrow tables, per-access checks, or rejection. |
| A-8 | Logical identity propositions. A handle relates pool identity, slot, generation or retirement proof, and resource version. Append-only handles omit generation; recyclable handles prove freshness through a library-chosen finite representation. | It preserves the no-tax distinction between append-only and reusable identities. Delete it and PF-G09 requires physical pointers, universal generation checks, or stale reuse. |
| A-9 | Proof-directed destruction. The owner's final state proof determines exactly which places and child roots are live and therefore destroyed. Destruction is generated from the proof-normal form; it cannot invoke arbitrary user finalizers. | It avoids full-capacity scans and double-drop while supporting partial construction and recursive owners. Delete it and partial storage leaks, scans capacity, or needs runtime tags. |
| A-10 | C0 in full. | Without C0, proof-indexed ordinary code still cannot create machine events, hide representations, call generic behavior, recover allocation, pin addresses, or discharge optimizer checks. |

### 3.3 Composition and library derivation

A dense sequence defines state `Prefix(len, cap)` and proves that each place below `len` is live. Push derives one vacant place, performs A-4 initialization, and returns `Prefix(len+1, cap)`. Growth first obtains a C0-7 root, proves two disjoint resource ranges, relocates each live place through A-5, and releases the empty old root. No spare slot is initialized and destruction ranges only over the live prefix.

A sparse table stores ordinary occupancy metadata and an indexed resource state whose finite-map proposition agrees with that metadata. A checked metadata branch produces one version-bounded occupancy proof; A-4 then authorizes the payload place. Dense and ring libraries use range proofs instead and store no bitmap.

W-GAP represents two live intervals and one vacant interval. A move shifts the interval boundary and relocates each affected owner once. W-PIPE represents each adapter's callable and source states as a product; monomorphized A-5 transitions inline without intermediate collections. W-POOL and W-GRAPH use A-8 logical handles. Shared owners use A-1 state plus C0-8 atomic events; ordinary libraries specify the strong/weak transition relation in A-5.

This route can derive fixed/AoS buffers, dense and ring sequences, sparse tables, arenas, recursive trees, entry APIs, drains, owning and borrowing iterators, composed adapters, refinement wrappers, logical pools, reference-counted owners, synchronization policies, channels, pinned state machines, and wrappers over exact external/target/image rows without standard-library privilege.

### 3.4 Expected implementation shape

- Parser and type checker: explicit state and proof parameters, affine proof/place values, pre/post signatures, and exact-use checking.
- Verifier: one fixed proof kernel for A-2, plus normalization and termination bounds. Solver output is an untrusted proof term.
- Compiler IR: indexed types and proofs erase; accepted place authority lowers to ordinary typed loads, stores, moves, and drops with alias scopes and bounds facts.
- Backend/runtime: no generic descriptor is required. Only representation-selected metadata and C0 runtime/target bodies remain.
- Diagnostics/AI: errors must identify the first missing proposition and a legal restructuring. Writers must construct proof terms or use taught proof-producing library patterns.

### 3.5 Open failures and falsifiers

- A-2 may not be decidable or predictable enough when finite maps, ranges, provenance, versions, and happens-before compose.
- Proof terms and state parameters may cause source, diagnostic, compile-time, monomorphization, or AI-generation blowup.
- Proof normalization may hide exponential behavior or make separate compilation impractical.
- Derived cleanup from an abstract proof may not have a unique, bounded, code-size-stable implementation.
- A hostile model showing an accepted uninitialized read, wrong-root borrow, duplicate drop, stale generation, or race falsifies soundness.
- A normal-frontend prototype exceeding the preregistered checker budget, or a held-out requiring unbounded annotations or proof search, falsifies feasibility.
- Structural accounting that adds metadata, branches, scans, or code growth to NT-FIXED, NT-P2, W-GAP, or W-PIPE falsifies the no-tax claim.

## 4. Candidate B: bounded place-and-topology kernel

### 4.1 Thesis

Expose a small closed algebra of tag-free storage topology, affine place transitions, scoped access footprints, and exact protocol closure. Ordinary libraries choose representations and compose the closed forms; they cannot define new proof predicates or topology rules.

This candidate trades some generality for a smaller checker and more predictable AI patterns. Its risk is that the supposedly orthogonal algebra misses an efficient future shape and grows by accretion.

### 4.2 Exact capability set

| ID | Capability and semantics | Why selected; deletion consequence |
|---|---|---|
| B-1 | Tag-free storage owner. It owns `cap` aligned places for T but asserts no T liveness. It is affine and can be acquired only through C0-7 or fixed frame formation. Merely owning it permits no payload read or drop. | It separates allocation from initialization and admits affine elements. Delete it and PF-G01/G02 requires full initialization, sentinel values, Option tags, or per-object roots. |
| B-2 | Closed topology-view algebra: Full, Prefix, Ring, Sparse, Product, and Hole. Each view is tied to one or more B-1 roots and a version. Prefix and Ring carry only their necessary counters; Sparse relates representation-selected control metadata to payload places; Product relates finitely many roots and ranges; Hole exists only inside a protocol. Views are affine, statically sealed, and erase except for counters or metadata demanded by the representation. | These six constructors cover the registered topologies without a universal bitmap. Delete one and its removal witnesses require a weaker topology with extra traffic/metadata or a new built-in. Make the algebra user-extensible and B becomes A without A's proof kernel. |
| B-3 | Fixed place transitions: initialize vacant, move out live, replace live and return old, swap proven-disjoint live, overlap-safe relocate range, clone into live or vacant destination under C0-2, and destroy live. Each operation consumes and returns the corresponding topology view atomically at the checker level. | These are the irreducible owner-traffic distinctions in PF-G03. Delete one and its registered operation pays extra transfers, allocation, whole-owner death, or hidden clone/drop. |
| B-4 | Scoped topology protocol. An operation may open an opaque owner into a topology view and enter a transition scope. Hole views cannot escape. Every normal exit must close to one declared public topology; abandonment either leaves an already valid base view or runs compiler-derived fixed cleanup. | It supports partial construction, drains, entry states, node split, and failure without a general pre/post logic. Delete it and B-3 cannot safely compose multi-step transitions. |
| B-5 | Closed footprint algebra: whole root, field, literal index, checked distinct-index set, checked split ranges, monotone cursor remainder, and finite product/sum over roots. Dynamic distinctness or occupancy checks produce one scope- and version-bounded footprint. | It covers the registered borrow/disjoint routes and produces scoped alias facts. Delete a form and the corresponding access requires per-use checks, runtime borrow tables, copying, or rejection. |
| B-6 | Borrow-carrying aggregate rule. A declared aggregate field may retain a borrow leaf only when its signature records one B-5 source relation. Moving or admitted relocation preserves the relation; invalidating mutation is rejected. Result provenance is a finite product or tagged sum over declared sources. | It closes BR-RESULT and BR-STORED without general propositions. Delete it and adapters, cached items, callable state, and reference-valued payloads need copies or callback-only APIs. |
| B-7 | Closed logical-handle forms: append-only index, generation index, retired index, and owner-scoped handle. Each form declares invalidation and exhaustion. Append-only form has no generation field or check. | It serves PF-G09 while preserving P2. Delete the append-only form and simple pools pay reuse tax; delete reusable forms and stale handles revive. |
| B-8 | Fixed shared-lifecycle forms: unique, strong/weak counted, dynamically borrowed shared, and synchronized shared. Transitions are closed tables composed with C0-8 events; library policy chooses representation and cycle handling. | It gives the checker a bounded interference vocabulary. Delete it and shared libraries need A-style user proofs, universal runtime checking, or trusted internals. |
| B-9 | Fixed cleanup synthesis. Each B-2 topology and B-4 protocol has a canonical live-place enumeration and cleanup order. C0-2 callables may be invoked only where the form's exact contract requires them. | It avoids proof-directed arbitrary cleanup and full-capacity scans. Delete it and partial states leak or demand user finalizers. |
| B-10 | Fixed fact schemas for bounds/ranges, topology membership, refinement, handle freshness, footprints, shared lifecycle, and selected code shapes. Only B operations, C0 checks, and the fixed verifier produce them. | It removes repeated checks and guards without A's general logic. Delete it and PF-G15 remains; add arbitrary schemas and optimizer authority becomes open-ended. |
| B-11 | C0 in full. | Delete it and the bounded storage algebra cannot manufacture allocation, concurrency, target, external, executable-image, callable, refinement, or pinning semantics. |

### 4.3 Composition and library derivation

A dense sequence is an opaque record containing B-1 storage plus a Prefix view whose only runtime state is `len` and `cap`. Push opens the owner, uses B-3 initialize at `len`, closes `Prefix(len+1, cap)`, and returns the offered value on C0-7 precommit failure. Growth relocates the live prefix exactly once and releases the empty root. A ring selects Ring, a hash table selects Sparse with its own control bytes, and an ECS selects Product over columns. None forces another topology's metadata.

W-GAP uses Hole with two live intervals and one vacant interval; B-9 statically fixes abandonment behavior. W-PIPE uses B-6 aggregate relations plus C0-2 callables and the monotone-cursor footprint, with no collection materialization. Logical pools use B-7. Shared owners use B-8 and C0-8. Refinement wrappers use C0-4 and B-10. External and target wrappers borrow B-5 ranges and call exact C0 rows.

This route derives the same library categories listed for Candidate A, but only when their state can be factored into B's closed topology, footprint, identity, lifecycle, and cleanup forms. A new representation shape is a finding against B, not permission for ordinary source to invent a rule.

### 4.4 Expected implementation shape

- Parser/type checker: a bounded set of topology, transition, footprint, handle, lifecycle, and protocol forms; flow-sensitive state only inside B-4 scopes.
- Checker: table-driven transition validation and exact-exit analysis, with no user proof language and no arbitrary theorem proving.
- Compiler IR: B-3 lowers directly to normal move/load/store/drop operations; views erase except for representation state; B-5 emits alias and range facts.
- Backend/runtime: C0 only, plus no generic topology descriptor. Sparse, generation, sharing, and pinning metadata appears only in selected representations.
- Diagnostics/AI: a closed taught pattern for each view and transition; errors name the missing view/footprint form.

### 4.5 Open failures and falsifiers

- Six topology forms may not cover a future efficient structure, especially compressed, probabilistic, persistent, device-resident, or crash-consistent state.
- Sparse's fixed control/payload relation may be too weak for robin-hood, cuckoo, compressed, or concurrent migration without adding forms.
- B-8 may accidentally bake one shared-lifecycle or memory-reclamation policy into the language.
- Fixed cleanup may duplicate code or fail for recursive and callback-dependent partial progress.
- A held-out requiring a new topology, footprint, lifecycle, or cleanup form falsifies claimed completeness.
- Any protected simple path acquiring another form's counter, tag, branch, load, call, or metadata falsifies no-tax.
- An accepted mismatched control/payload version, escaping Hole, wrong-root stored borrow, duplicate destruction, or interference-stale fact falsifies soundness.

## 5. Candidate C: closed family-specialized substrate

### 5.1 Thesis

Provide separate compiler-known storage and protocol families for the performance shapes actually admitted by the workload ledger. Each family has its own representation contract, operations, cleanup, borrow rules, and fact production. Ordinary libraries wrap and combine these families but cannot define a new low-level family.

This candidate minimizes general proof machinery and makes generated code predictable family by family. Its risk is language-surface growth, compiler/backend special cases, and failure to generalize beyond the frozen witnesses.

### 5.2 Exact capability set

| ID | Capability and semantics | Why selected; deletion consequence |
|---|---|---|
| C-1 | Fixed affine aggregate family: contiguous Full storage of Copy or affine records, field projection, whole-record move, and exact field destruction. | Required by PF-G01 and AoS/FFI-shaped rows. Delete it and AoS requires SoA, per-object roots, or rejection. |
| C-2 | Dense sequence family: tag-free capacity plus one live prefix, with push/pop/insert/remove/replace/swap/relocate/grow/truncate/clear/clone and exact failure/cleanup contracts. | Required by W-SMALL, builders, heaps, and dense held-outs. Delete it and these paths pay full initialization, tags, rebuilds, or extra traffic. |
| C-3 | Ring family: wrapped live interval, two-slice exposure, front/back operations, make-contiguous, and exact wrapped destruction. | Required to avoid O(n) front removal without sparse metadata. Delete it and PF-G02's ring contract regresses. |
| C-4 | Sparse family: representation-selected control bytes or bitmap plus payload storage, occupied/vacant entry protocols, insert/remove/rehash, exact live-slot destruction, and versioned access facts. | Required by hash/slab/store contracts. Delete it and sparse structures use dense search, per-element allocation, or unsafe control/payload coupling. |
| C-5 | Dependent-product family: finitely many related roots/ranges with atomic migration, split, merge, and failure contracts. | Required by ECS, B-tree nodes, and multi-buffer transactions. Delete it and columns copy/rebuild independently or need a general proof system. |
| C-6 | Hole-transition family: compiler-known gap, drain, entry, partial-construction, relocation, and node-rebuild protocols, each with fixed exact-use or cleanup behavior. | Required by W-GAP and direct traffic. Delete it and family operations simulate with swaps, copies, or allocation. |
| C-7 | Borrow/access protocol families: returned element/slice, checked multi-index access, split ranges, entry guards, monotone cursors, owning cursors, and composed traversal. Each has explicit result provenance and invalidators. | Required by PF-G04 and W-PIPE. Delete a protocol and its operation needs copying, callback-only APIs, runtime tables, or a new special case. |
| C-8 | Refinement families: byte/text and declared protocol-value wrappers with fixed validators and invalidating mutations. | Required by PF-G08. Delete it and validation repeats or storage copies; generalize it and C approaches A/B. |
| C-9 | Identity families: append-only pool, recyclable generation pool, graph handle, and address-stable owner. Each has fixed representation and invalidation rules. | Required by PF-G09/G11. Delete a family and the corresponding witness pays indirection, universal checks, or cannot be expressed. |
| C-10 | Shared and concurrent families: strong/weak owner, dynamic shared cell, mutex/rwlock/condition variable, channel, and task/suspension families over C0-8. Each has fixed lifecycle and interference rules. | Required by PF-G10/G11 without a general shared-state logic. Delete them and concurrency requires trusted libraries or workload exclusion. |
| C-11 | Family-specific fact producers and code-shape guarantees. Each operation has a closed list of bounds, alias, state, identity, refinement, effect, and dispatch facts plus invalidators; no cross-family fact is inferred without an explicit composition rule. | Required by PF-G15. Delete it and checks/guards remain; make it implicit and cross-family optimizer mistakes become likely. |
| C-12 | C0 in full, including opaque wrappers and stateful callable contracts. | Delete it and family operations lose exact machine events, direct behavior, abstraction, refinement, failure algebra, and verified fact handling. |

### 5.3 Composition and library derivation

Ordinary libraries build named containers by wrapping one or more family owners. A vector-like library wraps C-2; a deque wraps C-3; a hash table wraps C-4; an ECS combines C-5 with identity C-9; a gap sequence uses C-2 plus C-6 only where the family contract admits the combination. W-PIPE uses C-7 adapters and C0-2 callables. Shared structures use C-10, exact machine events from C0-8, and family-specific payload storage.

The compiler family is not necessarily a standard-library container. It is a representation and protocol substrate available with no standard library. Nevertheless, ordinary libraries cannot derive an efficient unlisted topology or protocol; they must combine listed families or seek a language extension.

### 5.4 Expected implementation shape

- Parser/type checker: one type and operation table per family, plus explicitly listed cross-family compositions.
- Checker: local family state machines and fixed provenance/cleanup rules; little or no general proof search.
- Compiler/backend: direct lowering and canonical optimized shapes per operation; the largest number of special cases and regression tests.
- Runtime: representation-specific metadata and C0 bodies. No universal descriptor, but many family implementations may duplicate machinery.
- Diagnostics/AI: shortest local rules and canonical patterns, at the cost of a larger vocabulary and more family selection decisions.

### 5.5 Open failures and falsifiers

- A held-out with an unlisted topology or a cheaper cross-family algorithm falsifies completeness and forces surface growth.
- Family operations may become named-container APIs disguised as language rules, preventing ordinary library innovation.
- Cross-family combinations may grow quadratically or produce inconsistent ownership, failure, and cleanup semantics.
- Backend guarantees may be target-fragile and turn the specification into an optimization manual.
- AI may select the wrong family or repeatedly convert between families, adding copies and allocations.
- Any family charging its metadata or branches to NT-FIXED, NT-P2, or an unrelated family falsifies no-tax.
- Any mismatch between separate family state machines that permits wrong provenance, stale facts, duplicate drop, or race falsifies soundness.

## 6. Gap coverage crosswalk

`paper route` means that the candidate contains a semantic route, not that the route is proved, implemented, or measured. `deferred table` means C0 gives an exact table form but its platform row inventory is unenumerated and receives no completeness credit.

| Gap | Candidate A | Candidate B | Candidate C |
|---|---|---|---|
| PF-G01 affine aggregate storage | A-1/A-4/A-9 | B-1/B-3/B-9 | C-1 |
| PF-G02 partial and structured live storage | A-1 through A-6/A-9 | B-1 through B-4/B-9 | C-2 through C-6 |
| PF-G03 place transitions and relocation | A-4 through A-6/A-9 | B-3/B-4/B-9 | C-2/C-4/C-6 |
| PF-G04 borrow and disjoint access | A-7 | B-5/B-6 | C-7 |
| PF-G05 sealing and state authority | A-1/A-3/C0-1/C0-5 | B-2/B-4/B-10/C0-1 | C0-1/C-11 |
| PF-G06 behavior and traversal | A-5/A-7/C0-2 | B-4 through B-6/C0-2 | C-7/C0-2 |
| PF-G07 failure-atomic commit | A-5/A-6/C0-6/C0-7 | B-3/B-4/C0-6/C0-7 | C-2/C-4/C-6/C0-6/C0-7 |
| PF-G08 refinement | A-2/A-3/C0-4/C0-5 | B-10/C0-4/C0-5 | C-8/C-11 |
| PF-G09 logical identity | A-8 | B-7 | C-9 |
| PF-G10 shared concurrency | A-1/A-2/A-5/C0-8 | B-8/C0-8 | C-10/C0-8 |
| PF-G11 address/suspension | C0-3 plus A state/protocol rules | C0-3 plus B-4/B-8 | C-9/C-10/C0-3 |
| PF-G12 external I/O/FFI | C0-9; deferred table | C0-9; deferred table | C0-9; deferred table |
| PF-G13 target operations | C0-10; deferred table | C0-10; deferred table | C0-10; deferred table |
| PF-G14 executable image | C0-11; deferred table | C0-11; deferred table | C0-11; deferred table |
| PF-G15 verified facts/code shape | A-2/A-3/C0-5 | B-10/C0-5 | C-11/C0-5 |

## 7. Candidate completeness status

All three sets are complete as capability hypotheses over the 15-row ledger: each names a semantic route and excludes no workload family. None is complete as a production design.

The following remain deliberately open for every candidate:

- a formal safety model and hostile counterexample search;
- exact target-profile enumeration for C0-8 through C0-11;
- exact rule and diagnostic counts;
- checker and compile-time bounds;
- structural cost derivations against every visible and held-out witness;
- measured performance and code size;
- AI construction and repair stability;
- concrete syntax, specification text, compiler/backend/runtime implementation, and migration.

Candidate comparison must therefore distinguish paper coverage from evidence strength. A general route is not automatically smaller, a closed route is not automatically faster, and a shared C0 category is not an enumerated machine-event basis.
