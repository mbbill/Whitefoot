# Finite authority and membership experiments

This safe-Rust experiment checks concrete finite certificates for storage range
ownership. It is not a Whitefoot language extension, a production acceptance path,
or a proof that a symbolic library implementation is admitted by today's compiler.
Its question is narrower: can splitting, constructing, extracting, joining, and
borrowing ranges account for the separating live sets without trusting an asserted
initialization bitmap?

Run `make check` in this directory. The parent experiment owns that invocation.
`model.rs` is the complete checker, independent occupancy/conservation oracle,
positive traces, and hostile controls. `make measure` prints reproducible structural
counts, not runtime timings. Build outputs stay under ignored `.build/`. Retire this
model when production semantic tests supersede its selection question; it is not a
second compiler to maintain indefinitely.

## Observations

The checker holds linear interval tokens of the form `Raw[lo, hi)` or
`Initialized[lo, hi)`, plus exclusive loans. Split consumes one token; join consumes
two adjacent tokens of the same kind; construction consumes one raw slot and one
available linear obligation; extraction returns that obligation. Token consumption
prevents reusing stale names. Releasing storage requires complete raw coverage and
no loans; finishing also requires returning every remaining linear obligation.

The independent oracle stores one live bit per concrete slot and compares complete
coverage, initialization, disjoint loans, and conservation against the count fixed
at the trace's entry. It never recomputes the expected total from a later checker
state. Cross-review caught and corrected that potential oracle-independence gap.

| Experiment | Result |
| --- | --- |
| All live sets at capacities 1 through 8 | 510 states constructed, checked, drained, and released |
| Same states representable by one circular window | 184 states |
| Every prefix failure point at capacities 1 through 32 | All 560 complete traces pass; unconsumed inputs and extracted values are returned |
| One wrapped backing with live `[6,8)` and `[0,2)` | Two simultaneous exclusive span loans accepted; duplicate and cross-range uses rejected |
| Capacity 4, live slots 0,1,2, keep 0 and 2 borrowed | Extract and reconstruct slot 1 accepted; old token invalid before and after reconstruction |
| Capacity 3 near neighbor | Remaining slots 0 and 2 **are** a circular window; this refutes the initial paper counterexample |

The 510/184 comparison is exact for these finite state sets. It is **not** a workload
frequency, a performance comparison, or evidence that a window plus ordinary
library occupancy state cannot implement a sparse collection. An initialized
`Option<T>` array represents a different physical live-set contract and is not
excluded by these counts.

Seven test groups also reject raw reads, duplicate exclusive loans, access outside
a loan's extent, extraction while borrowed, stale tokens after split/join/extract,
invalid split endpoints, reversed or mixed-state joins, reuse after release, early
release of live elements, and unfinished linear responsibility.

## Concrete helper and evidence burden

The experimental prefix helper accepts exactly:

```text
Prefix(capacity=N, length=k,
       initialized=Initialized[0,k) if k>0,
       spare=Raw[k,N) if k<N)
plus one available owned input, with k<N
```

Its certificate splits the raw tail at `k+1` when needed, constructs the one raw
slot, joins it with the initialized prefix when needed, and returns the new
summary. `prefix_valid` derives that summary from the live tokens and their exact
endpoints; it does not accept a library's assertion. Mutating the ranges invalidates
the old summary. Rejected preconditions occur before mutation.

For a concrete fill of capacity N, the explicit interval construction uses `3N-2`
primitive transitions. A partial fill of k elements, `0<k<N`, uses `3k-1`. Coalescing
keeps construction at no more than three live range tokens. These are **unrolled
certificate execution counts**, not source annotation counts or a requirement to
write N proofs. The generic helper body has three kinds of range step and two
endpoint cases. A native prefix transition could hide all three steps behind one
compiler-owned operation. This experiment cannot price symbolic loop proof,
contract inference, or diagnostic quality for either production alternative.

The cleanup trace deliberately expands ranges into individual tokens so it can
check every linear extraction against the per-slot oracle. Its temporary checker
metadata must not be read as a proposed runtime container layout. None of these
tokens is demonstrated to survive or erase through a real backend here.

## What this changes and what it does not

- A single circular initialization window does not cover every required topology.
  Finite range authority can describe more without adding a new state-machine
  family for each concrete mask. This earns an internal representation experiment;
  it does not alone select a public generalized proof interface.
- Empty backing authority can be discharged independently of already-consumed
  linear payload obligations. The lifecycle source suite shows why a corresponding
  language rule is needed; merely adjusting code generation cannot change the
  current source rejection.
- Two-span borrowing does not inherently require a second payload allocation.
  The model checks its range relationships, not the emitted cost of a future view
  operation or its interaction with all existing borrow rules.
- The prefix wrapper rejects all live loans, including disjoint prefix loans.
  That is a conservative limitation of this helper, not a necessary restriction
  of interval authority. The separate sparse trace demonstrates disjoint access
  composition at the primitive level.
- Token identity is scoped to one checker/storage instance. This model does not
  verify returning a block to its original pool, multi-allocation provenance,
  typed payload identity, arbitrary abstract invariants, or stable object handles.
  Linear payloads are anonymous obligations; their data and destructor effects
  are outside the model.
- Choosing a construction stop point enumerates the owned state at each real
  failure prefix. It does not establish atomic recovery from a side-effecting
  constructor, asynchronous cancellation, or any retirement schedule.

Consequently, these results support separate initialization state and ownership
obligations, plus composable finite ranges as an implementation basis to evaluate.
They do not establish that checked library representation is cheaper to implement,
easier to write, or faster than compiler-owned state machines. That decision also
needs the real source results and the native dense-storage measurements.

## Two indexes and object lifetime

`membership.rs` is a separate, sequential native protocol control. It asks which
behavior a runtime-checked stable identity actually promises; it is not an
extension of `model.rs`'s certificate checker or a proposed WF implementation.
Its existing `make check` caller builds the safe-Rust model and its two tests.
Retire it when a checked source protocol and its maintained tests supersede this
contract comparison. It was checked on 2026-09-08 with Rust 1.98.1.

Each slot holds a boxed four-word payload. Handles contain store identity, slot
and finite generation. Two key indexes refer to the same object. Under weak
membership, deletion succeeds and later index lookup returns `Expired`, including
after the slot is reused. Under retained membership, deletion returns
`BusyMemberships(2)`, then `BusyMemberships(1)` as the indexes unlink. A separate
affine logical access ticket makes deletion return `BusyAccess(1)` until that
ticket is returned. These are different application contracts, not interchangeable
implementations of one retained reference.

Thirty-one fixed observations check lookup, capacity refusal returning its
unconsumed payload, both membership policies, access retirement, wrong-store
rejection, slot reuse and generation exhaustion. With a deliberately tiny maximum
generation of two, the slot retires after generations 0, 1 and 2 rather than
wrapping. An independent drop ledger observes all nine payload resources consumed
exactly once. A second test checks that the finite generations never repeat.

Review exposed a resource-loss bug in the candidate: returning a ticket to the
wrong store consumed the only ticket and left the original object permanently
busy. The corrected error carries `(error, original_ticket)`; a retained control
then returns that same ticket to the correct store and successfully deletes the
object. Refusal needs an ownership return contract even when runtime validation
itself is acceptable.

This candidate's metadata is sufficient only for its bounded operations; it is
not a measured minimum. The ticket is a logical deletion guard, **not a memory
borrow or backing keepalive**, and can outlive a dropped store. A separate
`borrowed_words` call returns a real safe-Rust reference tied to `&Store`; its
lifetime safety comes from Rust. Caller-selected distinct IDs exercise a
wrong-store check but supply no uniqueness authority for a real store factory.
Ticket abandonment, access-counter saturation, duplicate-link/refcount policy,
store-ID reuse, parallel readers, address relocation and deferred reclamation
remain outside the executed scenarios. The drop ledger's `Rc<RefCell<...>>` is
test instrumentation, not the candidate lifetime protocol.

No WF acceptance, stored-borrow design, pointer-sized handle, timing or concurrent
reclamation claim follows. The useful result is the contract distinction and the
need to retain access authority on a rejected transition. Optional slots or
initialization permissions alone do not provide either membership policy.
