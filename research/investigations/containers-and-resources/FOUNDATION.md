# Container performance ceiling and the foundation decision

The question is how Whitefoot can support efficient system containers while
preserving machine-checked safety. Performance has priority over breadth;
common containers are a basic coverage requirement, and kernel, database and
cache-server cases test where a representation ceiling would exclude useful
applications. A lower-level interface is a candidate means, not an objective.

The owner clarified this ground on 2026-09-08: inability to eliminate a runtime
check is not by itself a failure, especially when validation is off the critical
path. Compare executable cost, including initialization, lookup, mutation,
allocation, movement, peak storage, tail work and necessary metadata. A compact
proof or a small kernel inventory is not a substitute for those results.

**The overall architecture selection is open.** The previous recommendation to
make finite nominal invariants and full-array conversion the next container-wide
foundation was not established by the completed workload coverage. Its pool
contract was synthesized for a controlled compiler experiment. The evidence
below preserves useful findings without treating that experiment as a demand
distribution or an implementation priority.

The [active specification](../../../spec/kernel-spec.md) defines accepted
programs. The [compiler guide](../../../compiler/README.md) owns implemented
capability. [REASSESSMENT.md](REASSESSMENT.md) records the merged owned-place work.
The [external study](EXTERNAL-WORKLOADS.md) owns pinned source observations, and
[representation experiments](../../experiments/container-representation/README.md)
own executable evidence. No source syntax, production compiler or runtime is
changed by this research.

## Ground and evidence

The constitution's P0, R0, W1 and D17 provide the selection grounds. Comparison
must distinguish four questions:

1. Does a complete operation have a correct current-language expression?
2. Is a rejected form excluded by the specification or blocked by the compiler?
3. Does the expression force expensive representation or work?
4. Does a competing mechanism actually remove that cost while preserving the
   same ownership, failure, lifetime and operation contract?

Use the established families as the coverage set, then test critical operation
chains. Current Whitefoot fixtures establish capabilities, not prevalence.
Pinned upstream source reveals actual contracts and plausible cost mechanisms,
not measured hotness. A native model is not a checked Whitefoot implementation.

Runtime work is classified by purpose and position:

- Algorithmic tests such as missing-key lookup, collision resolution and format
  decoding remain necessary unless a stronger input contract removes them.
- Validation may reject malformed external bytes or a mismatched view, then
  expose a checked domain for subsequent work. The false edge is ordinary
  behavior; a one-time validator is not free if objects are short-lived.
- A source guard can establish the proof required by an operation on its true
  branch. This does not grant arbitrary raw bytes valid-T status or revoke loans.
- Pure proof bookkeeping erases. Existing required static proof is not replaced
  by a trap or an impossible-case error arm added merely to rescue a program.
  Any different checked-API rule is an explicit candidate amendment.
- Compare retained checks where they execute. Cache misses, layout, allocation
  and algorithm choice can dominate a predictable branch. Conversely, a repeated
  scan or metadata check can be material. Neither conclusion is assumed.

A preliminary native scalar-map control now compares the same linear-probing
algorithm with interleaved tags, separate control/payload, a retained per-access
extent check, and batch extent validation; a Rust standard-map comparator uses
the same keys, hash outputs and results with its different algorithm.
[Source, samples and limits](../../experiments/container-representation/costs/RESULTS.md)
separate representation effects from compiler acceptance and do not establish a
generic-map or system-wide performance winner.

## Coverage and discriminating contracts

This table is a research frontier, not a list of completed library implementations.
Each row needs a checked operation witness and, where cost selects between forms,
a matched executable comparison before it can support a performance conclusion.

| Family | Critical operation chain | Current route to test | Ceiling or missing evidence |
| --- | --- | --- | --- |
| Full arrays and fixed sequences | Construct non-copy elements, index, replace, consume, clean up | Full flat arrays; general-element fixed runs | Full-array element restriction; mandatory window metadata; final-place construction |
| Growable vector and strings | Reserve, append, refuse without losing input, relocate, drain | Store-backed run with source-written allocation and movement | No current realloc row; initialization/copy costs; general helper contracts |
| Deque and ring | Both ends, wrap, two-span processing, grow/rebase | Existing circular window | Two-span views and helper provenance; extra work when a consumer needs contiguous data |
| HashMap and HashSet | Collisions, duplicate insertion, lookup, delete, reuse, rehash | Initialized optional entries; initialized byte/control and copy-payload alternatives | Ordinary complete trace and generic payload/behavior coverage; sparse layout cost |
| Ordered maps/sets and priority queues | Search, range, insert/delete; sift/split/merge/rotate | Dense heap; recursive owning boxes; fixed-capacity node arrays | Dynamic disjoint access, mutable traversal, non-copy movement and full operation evidence |
| SmallVector and short strings | Inline use, spill, refuse, retain or shrink | Enum of inline and store-backed owners | Tag/layout, store-region and ABI cost; no completed matched spill implementation |
| Lists, sparse sets and stable slots | Remove by identity, reuse, preserve other identities | Indexed owners or recursive boxes | Index validation/generation costs; multi-membership; retained borrowing |
| Packed records and byte pages | Decode, insert/delete, overlap move, compact, validate | Initialized byte arrays/runs and codecs | Bulk lowering, compact handles, validation reuse; variable-sized inline records |
| Intrusive kernel structures | Link an externally owned object into multiple relations, unlink | Owning containers or IDs are possible different contracts | Stored membership references, stable placement and reclamation are independent needs |
| Shared/concurrent containers | Publish, observe, mutate, retire, reclaim | Existing staged lexical access covers only its stated scope | RCU/epoch/shared lifetime is not supplied by a sequential slot API |

The first [current-language family witnesses](../../experiments/container-representation/families/RESULTS.md)
now execute a bounded optional-entry hash table, a dense binary heap, and a
B+ tree leaf-split component. The hash trace does not yet cover growth or rehash;
the leaf component is not a complete ordered map. A separate valid-source boxed
tree reproducer exposes descriptor-replacement lowering failure and is explicitly
deferred executable correctness evidence, not a source-language rejection.

The binary heap also has a same-algorithm native comparison and an independent
sorting oracle. It exposes retained complete-run transfers at ordinary helper
boundaries despite the earlier fresh-destination improvements. That is a measured
implementation cost to investigate before attributing dense-heap performance to
the absence of a lower-level storage language. The helpers explicitly preserve
the contiguous head-zero property through verified contracts.

The Linux, Redis and SQLite observations in the external study make the last rows
concrete. They do not impose pointer tagging, GC, C callbacks or any upstream
threshold as a Whitefoot requirement. A byte-page codec does not need arbitrary
typed holes merely because its physical entry lengths vary. A borrowed cursor or
multi-index object cannot be claimed covered by copying values into a run.

Generic behavior is a separate axis: hash/equality/comparison and callbacks must
have an admitted invocation mechanism and effects. A concrete u64 table does not
establish a reusable arbitrary-key library. Memory safety also differs from
ordinary map correctness: fully initialized indexed storage can be memory-safe
without proving the entire abstract map algorithm. Prove additional semantic
properties when a contract or partial-operation domain actually needs them, and
test behavior independently.

## Competing foundation strategies

| Candidate | Possible advantage | Discriminating failure |
| --- | --- | --- |
| Current runs, views and ordinary library code | Small implemented safety basis; cheap range facts; usable byte and dense algorithms | An important matched operation forces extra tags, copies, scans, allocation or inaccessible lifetime |
| General compiler-checked state/layout operations | Can support full values, initialization destinations, compact optional layouts or checked ranges without exposing an arbitrary resource logic | New operation for each container; closed layout inventory prevents an important representation; compiler complexity without measured benefit |
| Library-selected typed storage with checked resource evidence | Allows library algorithms to choose initialized sets, placement and transitions | Dynamic state cannot be checked compositionally, proof effort explodes, erasure adds metadata, or borrowing/cleanup remains inexpressible |
| Checked runtime validation with reusable access | Can move proof work to construction/boundary paths while keeping hot operations simple | Validation repeats on hot mutation or cannot establish the needed ownership/initializedness authority |

These strategies can supply different components of one eventual design. No
hybrid is selected merely because it sounds flexible. Every component needs an
operation and cost that justify it; do not build all four infrastructures.

### A narrower competitor to arbitrary storage permissions

A general projected layout for ordinary slot enums deserves a direct comparison
with library-selected resource proofs. Conceptually, a slot has `Empty`,
`Deleted`, or `Occupied(key, value, hash)` state. Its authoritative discriminant
can be stored separately from payloads while each logical slot remains one valid
enum. Normal construction, matching, replacement and destruction could preserve
the safety relation. Replacing an occupied slot with a tombstone transfers the
old payload exactly once. Read-only control projection could support probing;
writing a byte alone must not create an occupied generic payload.

This is a hypothetical layout facility, not existing source syntax or a selected
implementation. It could obtain sparse typed storage without requiring writers
to prove an arbitrary ownership-set predicate. It has concrete limits:

- A tag plus an arbitrary byte fingerprint and extra empty/deleted states do not
  fit one byte. A compact control encoding needs a checked finite range/variant
  layout or pays extra metadata. A second occupancy bitmap is not presumed free.
- Projection does not imply safe simultaneous mutable access to arbitrary slots.
  The borrowing rules must admit the actual operation and conserve exclusivity.
- Ordinary insertion receives a complete value. Eliminating a fallible producer's
  large temporary still needs result-destination routing or checked construction.
- A fixed enum layout does not cover arbitrary mixed-type overlays, compact
  variable-tail objects, stored memberships or deferred reclamation.

For the broader resource-proof candidate, the decisive missing mechanism is
symbolic focus and framing: open the permission for a runtime-selected slot while
retaining responsibility for every other live slot. A finite list of concrete
tokens or enumerated examples is insufficient for arbitrary runtime capacities.
Allocation identity must distinguish two allocations in the same store region;
splitting evidence cannot duplicate it. Explicit finite proof terms with a fixed
resource grammar and checked induction are a candidate, but this is new proof
machinery rather than a widening of numeric `ensures` clauses. Its checking and
erasure have not been implemented or validated by the current finite model.

### Runtime-checked identity and retained membership

A stable slab with two indexes has two meaningfully different contracts. Under
weak identity, deleting an object may leave an index entry whose later lookup
returns `Expired`. Bounds, owner identity, generation and state checks can make
that access safe; removing stale index entries remains application correctness.
A finite generation must retire/refuse on exhaustion or justify safe reuse, not
wrap and silently revive an old identity. A generation alone does not prevent a
handle being applied to the wrong store.

Under retained membership, an object must remain the same accessible object while
the membership exists. Returning `Expired` after deleting it changes the
contract. Deletion must be statically prevented, delayed, or return an intended
`Busy` outcome through an authoritative lifetime protocol. Neither projected enum
validity nor a live-slot permission alone supplies that retained lifetime. A
runtime check also cannot preserve memory after it returns a borrow unless the
access protocol keeps the backing alive. These candidate identities are not a
claim that the arena-index ownership pattern rejected in STOR-1 has been admitted.

The next comparison should use the same non-copy payload in a deleting/rehashing
map and a slab with two indexes, testing weak identity and retained membership
separately. Price current complete-value storage, projected enum layout, and
resource evidence against the same operations, including intended runtime
checks. If a narrower representation matches the broader candidate's costs on
both contracts, generality alone does not justify a new resource logic. Conversely,
duplicate metadata, forced payload copies or an unrepresentable retained
lifetime can supply a concrete reason to widen the foundation.

### What a lower-level candidate must actually expose

A useful candidate separates allocation identity and extent, typed positions,
initialization/ownership responsibility, loans and executable cleanup.
Reservation gives storage, not a valid value. A position can form an ordinary
borrow only when its content is valid and the appropriate permission is held.
Split/join operations conserve exact ownership; reuse cannot resurrect old
permissions. Address stability lasts through the actual last use, including
staged join return, result consumption and retirement.

The hard new part is a checked abstraction over these resources. Separate
hash-control bytes cannot authorize reads of arbitrary uninitialized payload
merely because a control byte says occupied. A checked relation must connect the
metadata to the live owner of the same allocation/slot, or the implementation
must retain an authoritative safe runtime representation such as optional values.
For copy payloads, initializing every slot is another candidate; measure its
initialization cost rather than assuming it is unacceptable.

Ordinary scalar requires/ensures are insufficient for arbitrary dynamic ownership
sets. Finite explicit resource split/join/open/close certificates and specified
induction are candidates, not current INV-1 or FN-9 capability. The checker must
finish deterministically, need no SMT, and check written steps rather than
rediscover a global mutation history. Proof erasure must not generate a token
table, extra control tags or scans. Algorithmic metadata and actual destruction
still execute.

A proposed slot capability does not itself provide compact variable-tail layout,
generic callbacks, membership in multiple containers, stored borrow provenance,
incremental rehash progress, lock protocols or deferred reclamation. These are
separate ceilings and cannot be silently deferred while claiming a universal basis.

## Construction without a public hole in T

Current internal aggregate-result destinations and eligible fresh-binding reuse
are implemented. A public vacant initialization destination is not. The current
boundary operation consumes a complete element; an optimizer's removal of a
temporary is not a source guarantee of construction in the final vacant slot.

The retained [large-result probe](../../experiments/container-representation/foundation/RESULTS.md#current-whitefoot-large-result-boundary)
shows a complete fallible result and payload surviving a retained producer
boundary. Two candidates remain: general internal result-destination routing, and
an explicit checked initialization destination. A field destination must remain
vacant, exclusive and address-stable across the producer's effects. Success
publishes a complete value; failure handles every initialized field and returns
the promised state. Returning an existing value may require a move, and failure
does not undo unrelated effects.

The [finite construction model](../../experiments/container-representation/foundation/RESULTS.md#finite-result-tree-construction-model)
tests concrete transfer and cleanup strategies, not a symbolic language extension
or an implemented new ABI. Original recommendations excluding a public
initialization permission were confined to the pool experiment and do not settle
the broader foundation choice.

## Empty backing across library and pool boundaries

The prior pool is a local capability witness. Its strengthened contract combined
fixed backing, failure-prefix cleanup, return of that same empty backing and a
full-owner conversion. This lifecycle was synthesized rather than observed as
a complete workload in the upstream application sample.

A fixed-capacity inner run preserves its capacity through its type and retains
its runtime length. Reading that length remains possible. Current boundary
operations do not transport incidental element measures through their implicit
slot. A direct scalar contract can describe each run-length transition; it
cannot by itself describe all nested element contents. Adding content models
or checked state evidence would be a substantive proof extension, not simply
remembering how often a helper was called.

This does not establish empty-state transport as the principal container gap.
Runtime admission of a block, an already-validated interface, a specialized
state or a richer contract are candidates under the actual pool contract.
Do not invent an impossible failure branch to mask a missing required fact.

### Specialized slot versus a checked source wrapper

The previous finite nominal-invariant proposal remains a local alternative to a
specialized empty-block owner. Neither has established container-wide priority.
A Box changes placement and possibly address stability; it does not prove empty
length. A wrapper preserving arithmetic facts also does not justify reading an
uninitialized generic payload.

[The original proposal at its recorded revision](https://github.com/mbbill/Whitefoot/blob/370bf3249d68d2aa635fb671af4446de636c6f8e/research/investigations/containers-and-resources/FOUNDATION.md)
retains its detailed transition and descriptor comparison. Its implementation
selection is superseded by this document; the executable evidence remains useful
under its stated conditions.

## Next implementation decision and unresolved work

The next production change is not selected yet. First complete current-language
critical-operation probes across the common families, and preserve exact rejected
forms beside accepted alternatives when they expose a meaningful boundary.
Then compare costs under the same semantic and resource contracts, including
retained runtime checks. Attribute a gap to algorithm, representation, proof
surface, allocation interface or lowering before proposing a change.

The strongest candidate interventions will be tested against at least one
different family so a pool-specific or hashmap-specific fix cannot silently
become the universal substrate. Required negative evidence includes wrong-slot
permission, borrow across relocation, cleanup after partial construction,
collision-chain deletion, stale identity and invalid encoded offsets where
applicable. This is a finite set of discriminating programs, not a requirement
to verify whole operating systems before advancing the compiler.

The research is ready for an implementation decision when the coverage matrix
has concrete dispositions, the decisive performance comparisons are executable,
and the recommended slice names both the operation it unlocks and the unresolved
ceiling it does not yet address. Passing CI alone cannot establish that condition.
