# Candidate C v0 Audit Baseline

Date: 2026-07-15

Status: Stage 0 complete for bounded paper audit. This file freezes an audit
interpretation; it adds no capability and selects no production design.

## 1. Freeze identity

- Controlling plan commit: `ab14962c96bcf382d0e2030457aab0b4fb76fdfa`.
- Controlling plan SHA-256:
  `3a46609f7eeaae559ae2a609c0275058c7933c533ad31d45b4442105b9412c7a`.
- Candidate source: `PERFORMANCE-FIRST-CANDIDATE-SETS.md`.
- Candidate source SHA-256:
  `8f729e7f1654e760df2aa93e9b493c4593973f3b2ed5292f134e5135fe3c8761`.
- Frozen scope: C0-1 through C0-11 and C-1 through C-12 exactly as the source
  states them. This baseline may narrow an audit interpretation to fail closed;
  it may not invent an operation, proposition, fact kind, runtime row, topology,
  cleanup rule, or composition.

## 2. C0 boundary

| ID | Frozen authority | Representation or event charge | Does not grant |
|---|---|---|---|
| C0-1 | Opaque generative modules hide representation and create distinct static owner identities. | No runtime state unless the chosen representation already stores it. | No storage liveness, access, transition, optimizer fact, or machine event. |
| C0-2 | Stateful generic callable contracts record captures, call cardinality, effects, leaf disposition, and result provenance; concrete calls are direct and monomorphized. | Callable environment and algorithm-required state only. | No purity, occupancy, payload access, duplicate/elided call, or provenance beyond the declared result relation. |
| C0-3 | Selective immovability ties one owner state to one physical root; suspension closes through declared cancellation/destruction. | Only representation state required by the selected pinned/suspended mechanism. | No universal pinning, heap indirection, or logical-identity-to-address equivalence. |
| C0-4 | A deterministic checked validator seals an exact root/value/version refinement until a listed mutation invalidates it. | The validation check and representation state selected by the wrapper. | No liveness, ownership, or optimizer fact except through C0-5. |
| C0-5 | A closed compiler-owned fact-kind table records proposition, root, region/version, producer, consumers, invalidators, transfer, and facts-off behavior. | Explicit checks remain; accepted facts erase unless their producer requires runtime state. | No writer assertion, new fact schema, unsupported check elision, or cross-root transfer not listed by the schema. |
| C0-6 | Recoverable effects distinguish precommit owner return, documented partial progress, success, and abort; every normal result states owner/cleanup disposition. | Only outcome state required by the observable contract. | No implicit rollback, leak, clone, cleanup, or conversion of abort into recovery. |
| C0-7 | Fixed rows acquire, resize, and release checked physical roots with exact size, alignment, failure policy, and transfer. | The root handle and allocator/OS events required by the selected row. | No arbitrary source-defined allocator contract, root subdivision relation, payload liveness, or hidden recoverable allocation. |
| C0-8 | A fixed thread/atomic event table plus one memory model defines spawn/join, atomic, blocking/wakeup, ownership transfer, and happens-before. | Exactly the selected atomic, fence, synchronization, and thread events. | No implicit ordering, reclamation policy, interference invariant, or synchronization on unique paths. |
| C0-9 | Platform profiles enumerate exact external I/O, resource, process, dynamic-provider, and FFI-frame rows. | Exactly the selected calls, partial-progress state, handles, and validation. | No generic syscall, arbitrary ABI, invented symbol contract, or unstated partial progress. |
| C0-10 | Target profiles enumerate exact scalar, SIMD, feature, MMIO, DMA, cache, ordering, reset, and fallback rows. | Exactly the selected machine/device events and target-required state. | No arbitrary opcode, semantic descriptor, unsupported feature, or optimizer fact outside C0-5. |
| C0-11 | Fixed image-lifecycle rows cover writable allocation, relocation, binding, permission transition, cache coherence, immutable identity, activation, quiescence, and release. | Exactly the selected image and permission events. | No simultaneous write/execute authority, arbitrary loader contract, or fact detached from final-image identity. |

C0 table forms are frozen but their platform row inventories remain
unenumerated. An audit requiring an unspecified row receives `C0-GAP`, not a
paper route.

## 3. Candidate C family boundary

| ID | Frozen family authority | Representation-charged runtime state | Boundary and forbidden inference |
|---|---|---|---|
| C-1 | Full contiguous affine aggregates: field projection, whole-record move, and exact field destruction. | Payload and padding required by the declared aggregate representation. | Full means every payload field is live. It cannot represent spare capacity, arbitrary holes, occupancy, or a partial drop set. |
| C-2 | Dense sequences with tag-free capacity and one live prefix; fixed dense operations and exact failure/cleanup contracts. | Length/capacity and allocation state selected by the representation; no per-place liveness tag. | Liveness is exactly the prefix. It grants no ring, sparse, arbitrary-hole, multi-root, or shared authority. |
| C-3 | Rings with one wrapped live interval, two-slice exposure, endpoint operations, make-contiguous, and wrapped destruction. | Head/length/capacity or an equivalent representation-required counter set. | It grants no arbitrary occupancy, interior hole, or sparse control/payload relation. |
| C-4 | Sparse representations with selected control bytes or bitmap, payload storage, occupied/vacant entry protocols, insert/remove/rehash, live-slot destruction, and versioned access facts. | Only the representation-selected sparse control state plus capacity/allocation state. | Control metadata grants payload authority only through a frozen C-4 rule tied to the same root relation and version. Hash/equality or an integer index grants nothing by itself. |
| C-5 | Finite dependent products of related roots/ranges with atomic migration, split, merge, and failure contracts. | Only counters/metadata required by each selected component representation and their declared relation. | It cannot define an arbitrary predicate, unbounded root graph, or cross-root provenance transfer not listed by the family. |
| C-6 | Fixed gap, drain, entry, partial-construction, relocation, and node-rebuild protocols with exact-use or fixed cleanup. | Only progress state required by the selected protocol; no universal hole tag. | A protocol cannot escape, invent a cleanup callback, or authorize a transition absent from its fixed state machine. |
| C-7 | Fixed returned-element/slice, multi-index, split-range, entry-guard, cursor, owning-cursor, and traversal protocols with explicit provenance and invalidators. | Only cursor/guard state required by the selected access protocol. | An index, callback result, aggregate field, or owner name does not establish provenance; each result leaf must name its source relation. |
| C-8 | Fixed byte/text and protocol-value refinement families with validators and invalidating mutations. | Only representation state selected by the refinement; validation may be runtime. | Refinement does not establish liveness, ownership, occupancy, or unrelated facts. |
| C-9 | Fixed append-only, recyclable-generation, graph-handle, and address-stable identity families. | Only identity/generation/address-stability state selected by that family. | Append-only paths pay no generation cost. Logical identity does not imply physical-root provenance or address stability. |
| C-10 | Fixed strong/weak, dynamic-shared, synchronization, channel, and task/suspension lifecycle families over C0-8. | Only reference, guard, synchronization, queue, or task state selected by that family. | One lifecycle form does not grant another's reclamation or memory-ordering policy; unique paths acquire no shared tax. |
| C-11 | Closed family-specific fact producers and code-shape guarantees with explicit invalidators and listed cross-family rules. | Explicit validation/check events when required; facts otherwise erase. | No implicit cross-family fact, writer-defined producer, unlisted transfer, or check elision without C0-5 acceptance. |
| C-12 | C0-1 through C0-11 in full. | Exactly the C0 row and representation charges listed above. | C-12 cannot fill an unenumerated C0 row or turn a machine event into an ordinary family operation. |

The family is a low-level representation or protocol substrate, never a named
container or project. A library may wrap and name a family; the compiler may not
recognize that library name to grant authority.

## 4. Frozen composition rules

Only these composition classes receive paper consideration. Each still needs an
operation-specific exact route.

| Composition | Permitted connection | Forbidden shortcut |
|---|---|---|
| C-2/C-3/C-4/C-5 + C-6 | A listed storage family enters a fixed transition protocol whose normal exits restore one listed valid family state. | An arbitrary escaped hole, user cleanup program, or protocol not listed by C-6. |
| C-2/C-3/C-4/C-5 + C-7 | A listed access protocol derives borrows from the exact live places and root relation of the storage family. | Deriving provenance from a container object, temporary guard field, integer index, hash, or callback frame. |
| Any C family + C-11/C0-5 | A listed operation may produce only its closed facts, bound to exact roots and versions, with listed invalidators. | Inferring a fact from a type/family name or transferring it across mutation, relocation, or roots without a rule. |
| Storage family + C0-2 | A behavior call may inspect or transform only authority passed by the family operation; the declared call relation accounts for callable state and results. | Treating hash/equality/comparison as pure, duplicable, or occupancy-authorizing without its contract. |
| Storage family + C0-6/C0-7 | Allocation and recoverable failure follow one listed commit point and exact owner disposition. | Hidden clone, leak, rollback, allocation, or changed failure contract. |
| C-10 + C0-8 | A fixed shared lifecycle selects exact thread/atomic events and interference invalidators. | Adding synchronization to a unique path or assuming an unlisted happens-before/reclamation rule. |
| C family + C0-9/C0-10/C0-11 | A checked family place/root may be consumed by one exact external, target, or image row. | Generic syscall/opcode/ABI/loader authority or facts outside the row contract. |

No other composition receives authority. Missing composition is
`COMPOSITION-GAP`; an unenumerated machine row is `C0-GAP`.

## 5. Family admission rule

A later report may propose, but not add, a family only when all conditions hold:

1. at least two independent frozen demands require the same irreducible
   low-level state machine;
2. all frozen existing-family compositions have a concrete safety failure or a
   named initialization, zeroing, payload traffic, allocation, metadata,
   indirection, check, synchronization, machine-event, code-size, or asymptotic
   tax;
3. the new rule is finite, deterministic, checker-local, and has exact normal,
   failure, abandonment, cleanup, provenance, and invalidation behavior;
4. it contains no project, container, library, API, path, or symbol identity;
5. unrelated programs acquire no field, tag, counter, branch, check, load,
   call, allocation, atomic, fence, machine event, or code-size tax; and
6. the proposal states whether it causes C to converge toward B's shared
   algebra or A's general proof authority.

One demand can expose a gap but cannot by itself admit a family.

## 6. Audit evidence and cost taxonomy

The only evidence states are `ROUTED`, `COMPOSED`, `FAMILY-GAP`,
`COMPOSITION-GAP`, `C0-GAP`, `TAXED`, `OPTIMIZER-GAP`, and `UNKNOWN`, with the
meanings fixed by the controlling plan. A row records every cost dimension:

- initialization/zeroing;
- payload copy/move;
- allocation;
- metadata;
- indirection;
- branch/check;
- scan;
- atomic/fence;
- machine event;
- code size; and
- asymptotic behavior.

Zero delta is a structural paper claim only. It is neither generated-code nor
measured-performance evidence.

## 7. Audit row schema

Each row has these required fields:

`project`, `revision`, `source_identity`, `operation`, `observable_contract`,
`reference_representation`, `live_state`, `owner_traffic`, `provenance`,
`failure_cleanup`, `fact_behavior`, `c_route`, `c0_dependency`,
`reference_events`, `forced_delta`, `evidence_state`, `falsifier`, and
`evidence_reference`.

One row represents one operation outcome or transition whose owner/failure
behavior differs. An audit may use several rows for one of the five frozen
operation names but may not exceed its plan row limit.

## 8. Universal safety and no-tax invariants

1. Vacant payload is never read, borrowed, moved as a value, or destroyed.
2. Every affine owner is live in exactly one place or disposed exactly once on
   every normal outcome.
3. Relocation ends source liveness before destination liveness and never reads
   or destroys a moved-from place.
4. Every borrow leaf names the exact physical root relation; mutation,
   relocation, root release, or version change applies its listed invalidator.
5. Control metadata, counters, identity, hash/equality state, and callable state
   grant no payload authority except through a frozen rule.
6. Failure and abandonment preserve one declared valid state or perform one
   exact fixed cleanup; abort grants no subsequent read.
7. Facts arise only through C-11/C0-5 and never relax a source-level check
   without machine verification.
8. Fixed/full/dense paths pay no sparse control state, generation, sharing,
   pinning, external, target, or image-lifecycle cost.
9. Unique paths emit no atomic, fence, blocking, wakeup, or reclamation event.
10. Unused families add no representation field, branch, check, load, call,
    allocation, machine event, or required code path.

Any violation is disqualifying; it is not averaged against an advantage.

## 9. Explicit unresolved definitions

The freeze makes the absence of these definitions unambiguous; it does not fill
them:

1. C-4 does not enumerate the admissible control-metadata-to-payload relations
   or the checker rule that validates one selected relation.
2. C-4/C-6 does not enumerate exact insertion, replacement, removal, and rehash
   state machines or their failure/abandonment cleanup.
3. C-4/C-7 does not enumerate entry/result provenance and invalidation for a
   selected sparse representation.
4. C-11 does not enumerate sparse occupancy, group-match, probe, or rehash fact
   schemas and invalidators.
5. C0-10 does not enumerate a Hashbrown-relevant portable group/SIMD row and
   fallback contract.
6. C0-7 does not enumerate the allocation row and failure behavior selected by
   a Hashbrown growth route.

Stage 1 must score these absences. It may conclude `COMPOSITION-GAP`, `C0-GAP`,
`OPTIMIZER-GAP`, or `UNKNOWN`; it may not silently complete them.

## 10. Stage 0 disposition

`STAGE-0-AUDIT-READY`.

The existing candidate can be frozen without adding authority because every
ambiguous capability is treated as absent rather than interpreted favorably.
This baseline is sufficient to score the exact Stage 1 operations. It is not a
claim that Candidate C is semantically complete. Stage 1 remains authorized and
must fail closed against the six unresolved definition classes above.
