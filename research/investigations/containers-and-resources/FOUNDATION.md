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

**Select ordinary valid values as the implementation baseline, and a projected
slot layout as the next storage-extension experiment.** A general library
resource-permission system remains a bounded alternative, not a selected public
foundation. The experiment must preserve the operation and ownership contracts
below without extra per-slot permission metadata; failure to do so reopens that
choice. This is a next-implementation decision, not a claim that current WF
covers every system container.

The earlier pool-driven recommendation of nominal invariants and full-array
conversion did not establish a container-wide priority. Its controlled contract
remains useful evidence, but neither it nor the new map controls measure demand
distribution. Common-family coverage and independent lifetime/layout ceilings
remain explicit below.

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
now execute a bounded optional-entry hash table, a dense binary heap, a
B+ tree leaf-split component, and a variable-record byte page with insertion,
deletion, overlapping movement and ordinary invalid-input/refusal outcomes.
An additional boxed-entry component now executes runtime-indexed migration and
collision probing, with earlier map operations prepared at selected positions.
It does not establish a general map API or return/resume migration contract;
the leaf component is not a complete ordered map. A separate valid-source boxed
tree reproducer exposes descriptor-replacement lowering failure and is explicitly
deferred executable correctness evidence, not a source-language rejection.

The binary heap also has a same-algorithm native comparison and an independent
sorting oracle. It exposes retained complete-run transfers at ordinary helper
boundaries despite the earlier fresh-destination improvements. That is a measured
implementation cost to investigate before attributing dense-heap performance to
the absence of a lower-level storage language. The helpers explicitly preserve
the contiguous head-zero property through verified contracts.

The [byte-growth comparison](../../experiments/container-representation/families/RESULTS.md#explicit-byte-run-growth-and-refusal)
executes source-written allocation, copy, fill and refusal, including injected
first/second allocation failure. Existing storage preserves the old bytes on
failed growth inside the trace. Matched loop, bulk-copy and realloc controls show
that growth policy and complete-operation cost must be measured: realloc loses
at the smallest tested size, while larger operations include generation and
digest work that can dominate copying. No spare-byte initialization is forced
by this source. Retained circular-index arithmetic and scalar copying provide
an ordinary lowering target; these timings do not select a new storage authority.

The Linux, Redis and SQLite observations in the external study make the last rows
concrete. They do not impose pointer tagging, GC, C callbacks or any upstream
threshold as a Whitefoot requirement. A byte-page codec does not need arbitrary
typed holes merely because its physical entry lengths vary. A borrowed cursor or
multi-index object cannot be claimed covered by copying values into a run.

The [permission-mechanism counterchecks](EXTERNAL-WORKLOADS.md#permission-mechanisms-as-design-counterchecks)
likewise separate a branded access discipline from backing lifetime, and raw
permission splitting from checked typed layout. Their proof/trust models are
comparators, not acceptance authority for WF.

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

These rows are not four mutually exclusive architectures. Runtime validation is
an access strategy available to each representation, and a projected enum is a
layout extension of ordinary valid values. The unresolved storage choice is
narrower: can compiler-maintained valid values with library-selected layouts meet
the required costs, or must libraries also compose initialization, borrowing and
release permissions? Improvements to ordinary place/ABI handling are relevant to
both. Do not build four infrastructures or choose the broader proof surface
because its claimed coverage is larger.

### A narrower competitor to arbitrary storage permissions

A general projected layout for ordinary slot enums deserves a direct comparison
with library-selected resource proofs. Conceptually, a slot has `Empty`,
`Deleted`, or `Occupied(key, value, hash)` state. Its authoritative discriminant
can be stored separately from payloads while each logical slot remains one valid
enum. Normal construction, matching, replacement and destruction could preserve
the safety relation. Replacing an occupied slot with a tombstone transfers the
old payload exactly once. Read-only control projection could support probing;
writing a byte alone must not create an occupied generic payload.

This is a selected experiment, not existing source syntax or a validated
implementation. It could obtain sparse typed storage without requiring writers
to prove an arbitrary ownership-set predicate. For R0, the expected W3/W1 delta
is for authors of custom container representations: obtaining compact layout
through checked ordinary value operations without unchecked implementation
steps. This is not a claim that using Rust's existing safe collections requires
unsafe code, nor a measured speedup over Rust or a demonstrated WF authoring win.
It has concrete limits:

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

### A bounded symbolic resource candidate

The broader route needs an actual checking rule for a runtime index, not a
compile-time token for every element. One candidate restricts predicates to
finite sums, separating products and structurally indexed segments. The notation
here is a research sketch, not Whitefoot syntax or implemented acceptance:

```text
Slot(ac, ap, i) = Ctl(ac,i,Empty)   * Raw(ap,T,i,i+1)
               | Ctl(ac,i,Deleted) * Raw(ap,T,i,i+1)
               | exists fp:checked7, v.
                   Ctl(ac,i,Full(fp)) * Init(ap,T,i,v)
Seg(ac,ap,lo,lo) = emp
Seg(ac,ap,lo,hi+1) = Seg(ac,ap,lo,hi) * Slot(ac,ap,hi)

focus(Seg(ac,ap,lo,hi), proof(lo <= i < hi))
  -> exists fresh k.
     Seg(ac,ap,lo,i) * Slot(ac,ap,i) * Seg(ac,ap,i+1,hi)
     * FocusKey(k,ac,ap,lo,hi,i)
```

`ac` and `ap` are fresh existential allocation identities produced by allocation;
they cannot be manufactured from an integer, equal capacity or a reused address.
`*` conserves linear responsibility. Focus generates a fresh linear closing key;
unfocus consumes the three pieces and that same key. It cannot recreate a segment
while a piece is borrowed or missing. Finite split/join, checked sum
introduction/elimination, construct/take,
borrow/end and release rules form a syntax-directed checker. Predicate definitions
cannot add axioms or arbitrary executable tests. A `Full` control byte by itself
never introduces an initialized `T`. The seven-bit fingerprint is ordinary
algorithmic data; spatial validity does not establish that it matches the key.

Opening consumes the selected `Slot` and its `FocusKey`, producing
`Open(k,ac,ap,i,control_state,payload_state)`, an exclusive linear resource retaining
that key and owning the slot's initialized control byte and raw or initialized
payload responsibility. Taking a payload changes its payload state to raw and
yields an ordinary owned value, leaving `Open(...,Full(fp),Raw)` until the control
is changed. Closing consumes the open resource and returns a valid `Slot` and
that same `FocusKey` only when its two states satisfy one of the alternatives above.
Until `Deleted` is written, the physical control can still say `Full`. That
transient state is safe only because the closed slot/segment authority has been
consumed: other code cannot observe it as a valid closed table. Every transition
updates the open responsibility, and returning a normal table requires restoring
the relation and closing it. Letting a shared closed-table view coexist with
this transition would permit an uninitialized read and must be rejected.

A written loop invariant can own a processed empty/deleted source prefix, its
remaining segment, the complete destination segment, and an optional separately
held entry. The checker verifies entry, one symbolic iteration and exit, with
explicit arithmetic steps and one structural fold/unfold. Runtime capacity does
not cause proof unrolling or occupancy enumeration. Migration focuses source
index `i`, extracts its entry, closes that source as deleted, then probes dynamic
destination indices. Each focus retains the entire complement. Successful
placement consumes the held entry; exhausted probing or intended refusal returns
both table owners, cursor and still-held entry as partial progress. It does not
promise rollback. A second simultaneous focus requires checked disjoint indices.

Cleanup uses the same induction to consume live values and recover vacant
coverage. `Ctl(ac,i,state)` owns one initialized byte. After payloads are retired,
cleanup unfolds and consumes vacant slots rather than rebuilding their segment.
A `retire_ctl` step consumes each control byte's responsibility and yields
`Raw(ac,u8,i,i+1)`; adjacent raw spans fold/join to cover `ac` without clearing its
physical bytes. Freeing a backing requires its complete raw coverage and no
remaining loan, initialized cell or open responsibility *for that allocation*. An extracted
Box may remain independently owned after the old slot backing is freed. Allocation
identity, segment/focus/open evidence and induction erase; actual control loads,
probing, payload movement, destruction and allocation execute. A runtime bounds
guard is possible only with an intended false outcome, not an injected trap.
No runtime proof table, second occupancy bitmap or additional cleanup scan is
part of this candidate.

The sketch's `ac` and `ap` name separate allocations. It does not yet cover two
typed planes inside one backing, which is the fair primary native layout
comparison: charging only the split representation for a second allocation would
mix layout with provider/refusal costs. One backing needs a checked layout rule
in addition to focus. It must consume raw bytes under one root allocation
identity, establish aligned, non-overlapping control and payload planes, and
retain padding responsibility and the root release authority. Plane identities
are not independently freeable allocations. Before freeing the root, all planes
and padding must return their complete raw byte coverage with no live loans.
Integer extent/alignment checks can occur in a total layout constructor with an
intended size-refusal result; this is different from revalidating ownership on
each payload access. This shared layout obligation applies to a projected enum
as well as to a library resource implementation. Ordinary byte splitting and a
cast do not establish typed validity, alignment or release authority.

This is a plausible restricted proof design, not a demonstrated sound checker,
erasure result or authoring-cost measurement. Checked, definition-justified
predicate introduction provides authority; privacy could hide representation but
is not what makes a proof unforgeable. The design covers fixed-stride sequential
storage. General zero-sized linear elements would still need counted ownership
obligations even when their byte extent is empty. It does not supply arbitrary
overlays, variable tails, hash correctness,
stored membership, concurrent mutation or backing keepalive. Ordinary enums
provide the control/payload validity relation; a projected-enum layout would
have to preserve it while admitting the required dynamic operations. If their
representation and operations match, this larger proof surface has no established
performance advantage. A concrete counterexample
to that narrower route remains the condition for selecting it.

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

A [two-index native control](../../experiments/container-representation/authority/RESULTS.md#two-indexes-and-object-lifetime)
now distinguishes those contracts through weak expiry, two retained memberships,
an independent access ticket, wrong-store rejection and finite generation
exhaustion. A rejected ticket return must preserve the ticket so it can still
release the original access. This experiment does not supply WF admission,
store-identity uniqueness or backing lifetime: its logical ticket is not a borrow,
and its separate actual borrow relies on Rust's lifetime rules. It confirms the
protocol distinction without pricing a proposed WF representation.

The matched comparison should use the same non-copy payload in a deleting/rehashing
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

## Evidence-backed implementation sequence

The next implementation experiment can be chosen without declaring a universal
container substrate. The current evidence supports this order:

1. Repair the existing owning-box replacement defect through the general borrowed
   owner descriptor path. The deferred ordered-node program must become an
   ordinary native success in all three modes. This restores a correctness
   baseline for owning-node algorithms; it does not add a new tree primitive.
   The reduced boxed-helper region-substitution discrepancy also blocks this
   nested-result helper composition: first repair FN-2 substitution through
   nested input/output types, then validate the rest of the deferred program.
2. Investigate ordinary owned-helper argument/result destinations using the heap
   cost control and the earlier dense and fallible-result controls. A move into a
   helper followed by return to the owner should not intrinsically require a
   payload copy at every call. The byte-growth control adds a separate loop/bulk
   lowering question: investigate whether contiguous-access and alias facts
   enable better ordinary loop/bulk lowering, then measure the same operation.
   Reuse must preserve RHS
   evaluation, reads of the old value, aliasing, failure cleanup and actual loan
   retirement. Re-measure
   the same operations after the change; the current ratio is not a promised
   speedup. This is a general lowering experiment, not a heap-specific ABI.
3. Prototype projected layout for ordinary slot enums against the native owning
   sparse control below. Keep construction, matching, transfer and cleanup on
   general valid-value operations. The target is one backing with compact control
   and payload planes, runtime-indexed access and no second occupancy/token table.
   Compare with the current ordinary representation and preserve the same refusal
   and migration outcomes. The resource-proof route remains a challenger with
   explicit symbolic checking and erasure obligations.

No production code is changed by this research. These steps identify useful
implementation experiments. The first two address executable compiler defects or
measured lowering costs; the third tests whether a general layout mechanism can
retain the native sparse representation advantage with WF authority.

### Sparse experiment contract and decision boundary

The [owning sparse control](../../experiments/container-representation/costs/RESULTS.md#owning-sparse-layout-and-migration)
executes the native operation chain, resource conservation, refusal, actual
growth, returned migration progress and cleanup. At 4096 slots on the measured
ABI, matched one-backing layouts use 24 versus 17 bytes per capacity unit;
separate resource allocations and descriptors are counted outside those figures.
Half-full same-capacity rehash plus digest has a lower split-layout median in
this run. The narrow timing does not establish general hash-table throughput.
These are physical and protocol targets for the next WF experiment, not evidence
that either proposed WF authority is already admitted.

Use one concrete map payload containing an owning resource. Execute collision
insertion, duplicate replacement, removal, lookup past a tombstone, reuse, growth
rehash and final cleanup. Specify allocation refusal and an intended stop during
migration, returning all still-owned state; do not assume rollback of completed
work. Compare the same capacity, key/hash, control encoding, migration policy and
failure contract across ordinary optional values, projected enums and resource
permissions. A seven-bit fingerprint with empty/deleted states is one matched
encoding; an arbitrary eight-bit fingerprint is a different layout comparison.

Entry relocation is permitted in this first contract. Its owning resource retains
its separate backing, but the key and owner descriptor may move. A future stable-row
comparison must retain an actual borrow of the row/descriptor, not infer that need
merely from an owning payload. Do not charge every map for address stability it
does not promise. Track control examinations, migration work, allocation count,
peak backing, initialized bytes and payload movement; timing claims apply only
to the operation actually measured.

The native control is the physical target, not a checked implementation of either
WF authority. The projected-layout prototype must then admit the same dynamic
operation chain and conserve every resource through refusal, returned progress
and cleanup. A separate fallible large-element insertion must compare complete
results, internal result destinations and any proposed vacant destination with
the same effects and cleanup. This belongs to the construction/placement
implementation experiment: the existing fresh-result control does not already
establish in-place generic map construction.

If projected enums and resource permissions produce the same representation and
operations, timing cannot distinguish their proof authorities. The broader route
must additionally demonstrate symbolic runtime-index focus, retention of the other
slots, transfer/cleanup and erasure without an extra token table or scan. A finite
concrete model is insufficient. Conversely, a required layout or retained access
that the narrower route cannot express is a real discriminator even if a scalar
lookup benchmark happens to match.

Two independent controls prevent the map from replacing the overall coverage
question. The byte-page witness supplies insertion, deletion and malformed
input behavior, and the growth trace prices one complete byte-run operation.
Bulk page movement and reusable validation costs remain distinct from that trace.
The two-index control distinguishes weak lookup returning `Expired` from retained
membership making deletion return `Busy`, including store mismatch, reuse and
generation exhaustion. It does not establish an efficient checked WF admission
or backing lifetime. Neither experiment requires implementing concurrent RCU
first or silently changing the current STOR-1 boundary.

### Independent ceilings and dispositions

The available evidence supports different next actions for different families:

| Area | Current disposition | Evidence that would change it |
| --- | --- | --- |
| Dense/fixed sequences and priority queues | Use ordinary valid values; correct and improve general storage transfer first | Matched operations still force material initialization, descriptor or movement cost after that repair |
| Full arrays of general elements | Distinct completed-value form remains useful; current flat-element restriction and linear-empty cleanup are language boundaries | A checked construction/consumption route and its actual representation, not an empty pool contract alone |
| Hash and ordered containers | Ordinary scalar operations and a boxed migration component work; prototype projected sparse layout next | Failure to preserve the native owning-map contract/cost, or a required operation beyond ordinary projected values; ordered mutation follows owning-box repair |
| Deques and rings | A wrapped logical-index trace works; a wrapped run is correctly refused as one contiguous view | A two-span consumer/growth trace that prices any required copying and admits the actual loans |
| Growable runs, strings and inline/spill forms | Source-written byte growth/refusal executes and has loop/bulk/realloc controls; no WF realloc or finished spill result | General owner-return helper, repeated reserve/spill and copy-heavy resize controls including peak storage and address validity |
| Packed byte records | Current initialized byte storage executes variable records and overlapping movement | Measured bulk/initialization/compact-handle cost, or an actually required typed layout that byte codecs cannot preserve |
| Stable slots, sparse sets and multiple memberships | The two-index native control distinguishes weak/retained memberships and logical access; bounds alone supply neither | Priced lookup/metadata, authoritative store identity and a checked source admission/backing-lifetime design |
| Shared/concurrent containers | Sequential storage selection grants no reclamation protocol | A separately specified publish/read/retire contract with the actual memory model and scheduling behavior |

Generic hash/equality/comparison invocation is independently unavailable under
FN-2/3/5: numeric and linearity bounds do not let a library call a supplied
behavior. A concrete key type is sufficient for the representation experiment,
but cannot certify a reusable generic library. Stored member provenance is
separately restricted by STOR-5. Allocation/resize and variable-tail layout need
their own provider and layout contracts. A slot permission alone supplies none
of these, and the architecture must not claim those system needs solved.

The decision has three explicit outcomes. Ordinary valid values remain the
baseline for families whose measured problems lie in lowering. Projected layout
is selected for a bounded prototype because compact sparse storage has a
concrete physical advantage without yet requiring a new writer resource logic.
General resource permissions become the preferred candidate only if a required
operation, layout or lifetime cannot be retained through ordinary/projected
values and the alternative supplies a credible deterministic checking and
erasure path. Equal native code cannot choose between proof authorities.

Research can therefore hand off to those implementation experiments without
claiming a universal container substrate. Generic behavior, complete ordered
mutation, two-span consumers, inline spill, full general arrays, stored lifetime,
variable tails and concurrent retirement remain named capability questions.
They are not silently counted as solved or prerequisites to fixing the observed
compiler defects. Reopen the selected route when one supplies a concrete
contract/cost counterexample; do not infer either universal coverage or universal
failure from the bounded map alone.
